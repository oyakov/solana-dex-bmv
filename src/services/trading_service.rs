use crate::infra::{Database, SolanaClient, WalletManager};
use crate::services::{GridBuilder, PnlTracker, PivotEngine, RebalanceService};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use metrics::{counter, gauge};
use tracing::{error, info};

pub struct TradingService {
    _settings: BotSettings,
    solana: std::sync::Arc<SolanaClient>,
    #[allow(dead_code)]
    database: std::sync::Arc<Database>,
    wallet_manager: std::sync::Arc<WalletManager>,

    pivot_engine: PivotEngine,
    grid_builder: GridBuilder,
    rebalance_service: RebalanceService,
    pnl_tracker: tokio::sync::Mutex<PnlTracker>,
}

impl TradingService {
    pub fn new(
        settings: BotSettings,
        solana: std::sync::Arc<SolanaClient>,
        database: std::sync::Arc<Database>,
        wallet_manager: std::sync::Arc<WalletManager>,
        pivot_engine: PivotEngine,
    ) -> Self {
        let grid_builder = GridBuilder {
            orders_per_side: settings.order_grid.orders_per_side,
            buy_channel_width: settings.channel_bounds.buy_percent,
            sell_channel_width: settings.channel_bounds.sell_percent,
            buy_volume_multiplier: settings.order_grid.buy_volume_multiplier,
            sell_volume_multiplier: settings.order_grid.sell_volume_multiplier,
        };

        let rebalance_service =
            RebalanceService::new(solana.clone(), wallet_manager.clone(), settings.clone());

        Self {
            _settings: settings,
            solana,
            database,
            wallet_manager,
            pivot_engine,
            grid_builder,
            rebalance_service,
            pnl_tracker: tokio::sync::Mutex::new(PnlTracker::default()),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            wallet_count = self.wallet_manager.count(),
            "Starting TradingService main loop"
        );

        let tick_interval = tokio::time::Duration::from_secs(
            self._settings.trading_tick_interval_seconds,
        );
        let recovery_delay = tokio::time::Duration::from_secs(5);
        let mut interval = tokio::time::interval(tick_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            interval.tick().await;
            counter!("bot_ticks_total", 1);
            if let Err(e) = self.tick().await {
                counter!("bot_tick_errors_total", 1);
                error!(error = ?e, "Error during trading tick; entering recovery delay");
                tokio::time::sleep(recovery_delay).await;
            }
        }
    }

    pub async fn tick(&self) -> Result<()> {
        info!("Trading tick starting");

        // 1. Housekeeping (wallet balances etc)
        self.rebalance_service.rebalance().await?;

        // 1a. Ingest recent trades for PnL tracking
        let last_trade_ts = self
            .database
            .get_state("pnl_last_trade_ts")
            .await?
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0);
        let last_trade_id = self
            .database
            .get_state("pnl_last_trade_id")
            .await?
            .unwrap_or_default();
        let trades = self.database.get_recent_trades(last_trade_ts).await?;

        if !trades.is_empty() {
            let mut tracker = self.pnl_tracker.lock().await;
            let mut newest_ts = last_trade_ts;
            let mut newest_id = last_trade_id.clone();

            for trade in trades {
                if trade.timestamp > last_trade_ts
                    || (trade.timestamp == last_trade_ts && trade.id > last_trade_id)
                {
                    tracker.record_trade(trade.side, trade.price, trade.volume);

                    if trade.timestamp > newest_ts
                        || (trade.timestamp == newest_ts && trade.id > newest_id)
                    {
                        newest_ts = trade.timestamp;
                        newest_id = trade.id.clone();
                    }
                }
            }

            if newest_ts != last_trade_ts || newest_id != last_trade_id {
                self.database
                    .set_state("pnl_last_trade_ts", &newest_ts.to_string())
                    .await?;
                self.database
                    .set_state("pnl_last_trade_id", &newest_id)
                    .await?;
            }
        }

        // 2. Fetch live market data
        let market_id = &self._settings.openbook_market_id;
        let market_data = self.solana.get_market_data(market_id).await?;

        // 3. Compute Pivot
        // Note: For MVP we might not have full historical trades from DB yet
        let pivot = self
            .pivot_engine
            .compute_pivot(&[], &[], Some(&market_data), 0) // Assume day 0 for now
            .await;
        gauge!("bot_last_pivot_price", pivot.to_f64().unwrap_or(0.0));

        let pnl_snapshot = {
            let tracker = self.pnl_tracker.lock().await;
            tracker.snapshot(market_data.price)
        };
        gauge!(
            "bot_pnl_realized_sol",
            pnl_snapshot.realized_pnl.to_f64().unwrap_or(0.0)
        );
        gauge!(
            "bot_pnl_unrealized_sol",
            pnl_snapshot.unrealized_pnl.to_f64().unwrap_or(0.0)
        );
        gauge!(
            "bot_position_net_sol",
            pnl_snapshot.net_position.to_f64().unwrap_or(0.0)
        );
        gauge!(
            "bot_position_avg_cost",
            pnl_snapshot.average_cost.to_f64().unwrap_or(0.0)
        );

        // 4. Check if we need to rebuild the grid
        if self.rebalance_service.should_rebuild(pivot) {
            info!(?pivot, "Rebuilding grid...");

            // 5. Build Grid
            let grid = self
                .grid_builder
                .build(pivot, Decimal::from(100)) // Using 100 as default total size for now
                .await;

            gauge!("bot_grid_levels_count", grid.len() as f64);
            info!(levels = grid.len(), "Grid constructed");

            // 6. Execute Grid Update & Emit Metrics
            let mut total_depth = Decimal::ZERO;
            for (idx, level) in grid.iter().enumerate() {
                let side_str = match level.side {
                    crate::domain::OrderSide::Buy => "BUY",
                    crate::domain::OrderSide::Sell => "SELL",
                };

                // Emit granular metrics for each level
                gauge!(
                    "bot_grid_level_price",
                    level.price.to_f64().unwrap_or(0.0),
                    "side" => side_str,
                    "index" => idx.to_string()
                );
                gauge!(
                    "bot_grid_level_size",
                    level.size.to_f64().unwrap_or(0.0),
                    "side" => side_str,
                    "index" => idx.to_string()
                );

                total_depth += level.size * level.price;

                info!(
                    side = %side_str,
                    price = %level.price,
                    size = %level.size,
                    "Scheduling grid order (Phase 1 simulation)"
                );
            }

            // 7. Performance Indicators (Mocked for now)
            gauge!("bot_active_depth_usd", total_depth.to_f64().unwrap_or(0.0));
            gauge!("bot_fill_rate_percent", 88.0); // Mock
            gauge!("bot_bundle_latency_ms", 42.0); // Mock
        } else {
            info!("Pivot stable, no rebalance needed");
        }

        Ok(())
    }
}
