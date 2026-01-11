use crate::domain::OrderSide;
use crate::infra::{Database, KillSwitch, SolanaClient, WalletManager};
use crate::services::{GridBuilder, PivotEngine, RebalanceService, RiskManager, RiskSnapshot};
use crate::utils::BotSettings;
use anyhow::Result;
use metrics::{counter, gauge};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

pub struct TradingService {
    _settings: BotSettings,
    solana: std::sync::Arc<SolanaClient>,
    database: std::sync::Arc<Database>,
    wallet_manager: std::sync::Arc<WalletManager>,
    kill_switch: std::sync::Arc<KillSwitch>,

    pivot_engine: std::sync::Arc<PivotEngine>,
    grid_builder: GridBuilder,
    rebalance_service: RebalanceService,
    risk_manager: RiskManager,
}

impl TradingService {
    pub fn new(
        settings: BotSettings,
        solana: std::sync::Arc<SolanaClient>,
        database: std::sync::Arc<Database>,
        wallet_manager: std::sync::Arc<WalletManager>,
        pivot_engine: std::sync::Arc<PivotEngine>,
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
        let kill_switch = std::sync::Arc::new(KillSwitch::from_settings(&settings.kill_switch));
        let risk_manager = RiskManager::new(settings.risk_limits.clone());

        Self {
            _settings: settings,
            solana,
            database,
            wallet_manager,
            kill_switch,
            pivot_engine,
            grid_builder,
            rebalance_service,
            risk_manager,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            wallet_count = self.wallet_manager.count(),
            "Starting TradingService main loop"
        );

        let tick_interval =
            tokio::time::Duration::from_secs(self._settings.trading_tick_interval_seconds);
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

        if self.kill_switch.is_triggered().await? {
            self.handle_kill_switch("manual trigger detected").await?;
            return Ok(());
        }

        let risk_snapshot = self.build_risk_snapshot().await?;
        if let Some(reason) = self.risk_manager.evaluate(&risk_snapshot) {
            let reason_text = reason.to_string();
            self.kill_switch.trigger(&reason_text).await?;
            self.handle_kill_switch(&reason_text).await?;
            return Ok(());
        }

        // 1. Housekeeping (wallet balances etc)
        self.rebalance_service.rebalance().await?;

        // 2. Fetch live market data
        let market_id = &self._settings.openbook_market_id;
        let market_data = self.solana.get_market_data(market_id).await?;

        // 3. Fetch recent trades from DB for VWAP
        let lookback_secs = self.pivot_engine.lookback_window_seconds();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let recent_trades = self.database.get_recent_trades(now - lookback_secs).await?;

        // 4. Compute Elapsed seconds for Seeded Pivot
        // We use the first cached trade to estimate how long we've been streaming
        let cached_trades = self.pivot_engine.cached_trades().await;
        let elapsed_seconds = if let Some(first_trade) = cached_trades.first() {
            now.saturating_sub(first_trade.timestamp)
        } else {
            0
        };

        // 5. Compute Pivot
        let pivot = self
            .pivot_engine
            .compute_pivot(&[], &recent_trades, Some(&market_data), elapsed_seconds)
            .await;
        gauge!("bot_last_pivot_price", pivot.to_f64().unwrap_or(0.0));

        // 6. Check if we need to rebuild the grid
        if self.rebalance_service.should_rebuild(pivot) {
            info!(?pivot, "Rebuilding grid...");

            // 7. Build Grid
            let grid = self
                .grid_builder
                .build(pivot, Decimal::from(100)) // Using 100 as default total size for now
                .await;

            gauge!("bot_grid_levels_count", grid.len() as f64);
            info!(levels = grid.len(), "Grid constructed");

            // 8. Execute Grid Update & Emit Metrics
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

            // 9. Performance Indicators
            gauge!("bot_active_depth_usd", total_depth.to_f64().unwrap_or(0.0));
            gauge!("bot_pnl_realized_sol", 0.0); // Mock
            gauge!("bot_fill_rate_percent", 88.0); // Mock
            gauge!("bot_bundle_latency_ms", 42.0); // Mock
        } else {
            info!("Pivot stable, no rebalance needed");
        }

        Ok(())
    }

    async fn build_risk_snapshot(&self) -> Result<RiskSnapshot> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let since_timestamp = now.saturating_sub(86_400);
        let trades = self.database.get_recent_trades(since_timestamp).await?;
        let mut net_pnl_usd = Decimal::ZERO;

        for trade in trades {
            let notional = trade.price * trade.volume;
            match trade.side {
                OrderSide::Buy => net_pnl_usd -= notional,
                OrderSide::Sell => net_pnl_usd += notional,
            }
        }

        let daily_loss_usd = if net_pnl_usd < Decimal::ZERO {
            -net_pnl_usd
        } else {
            Decimal::ZERO
        };

        let market_id = &self._settings.openbook_market_id;
        let mut open_orders = 0u32;
        for wallet in self.wallet_manager.get_all_wallets() {
            if self
                .solana
                .find_open_orders(market_id, &wallet.pubkey())
                .await?
                .is_some()
            {
                open_orders = open_orders.saturating_add(1);
            }
        }

        Ok(RiskSnapshot {
            daily_loss_usd,
            open_orders,
        })
    }

    async fn handle_kill_switch(&self, reason: &str) -> Result<()> {
        counter!("bot_kill_switch_trigger_total", 1, "reason" => reason.to_string());
        info!(%reason, "Kill switch triggered; canceling all orders and pausing trading");
        self.cancel_all_orders().await
    }

    async fn cancel_all_orders(&self) -> Result<()> {
        if self._settings.dry_run.enabled {
            info!("Dry run enabled; skipping cancel-all execution");
            return Ok(());
        }

        let market_id = &self._settings.openbook_market_id;
        let tip_lamports = self._settings.jito_bundle.tip_lamports;
        let jito_url = &self._settings.jito_bundle.bundler_url;

        for wallet in self.wallet_manager.get_all_wallets() {
            let result = self
                .solana
                .cancel_all_orders(market_id, wallet, jito_url, tip_lamports)
                .await?;
            info!(wallet = %wallet.pubkey(), %result, "Cancel-all submitted");
        }

        Ok(())
    }
}
