use crate::domain::OrderSide;
use crate::infra::{DatabaseProvider, KillSwitch, PriceAggregator, SolanaProvider, WalletManager};
use crate::services::{
    FinancialManager, FlashVolumeModule, GridBuilder, PivotEngine, PnlTracker, RebalanceService,
    RentRecoveryService, RiskManager, RiskSnapshot,
};
use crate::utils::BotSettings;
use anyhow::Result;
use metrics::{counter, gauge};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

pub struct TradingService {
    _settings: BotSettings,
    solana: std::sync::Arc<dyn SolanaProvider>,
    database: std::sync::Arc<dyn DatabaseProvider>,
    wallet_manager: std::sync::Arc<WalletManager>,
    kill_switch: std::sync::Arc<KillSwitch>,

    pivot_engine: std::sync::Arc<PivotEngine>,
    grid_builder: GridBuilder,
    rebalance_service: RebalanceService,
    risk_manager: RiskManager,
    pnl_tracker: tokio::sync::Mutex<PnlTracker>,

    flash_volume: FlashVolumeModule,
    financial_manager: FinancialManager,
    rent_recovery: RentRecoveryService,
    price_aggregator: std::sync::Arc<PriceAggregator>,
}

impl TradingService {
    pub fn new(
        settings: BotSettings,
        solana: std::sync::Arc<dyn SolanaProvider>,
        database: std::sync::Arc<dyn DatabaseProvider>,
        wallet_manager: std::sync::Arc<WalletManager>,
        pivot_engine: std::sync::Arc<PivotEngine>,
        price_aggregator: std::sync::Arc<PriceAggregator>,
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

        let flash_volume =
            FlashVolumeModule::new(solana.clone(), wallet_manager.clone(), settings.clone());
        let financial_manager =
            FinancialManager::new(solana.clone(), wallet_manager.clone(), settings.clone());
        let rent_recovery =
            RentRecoveryService::new(solana.clone(), wallet_manager.clone(), settings.clone());

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
            pnl_tracker: tokio::sync::Mutex::new(PnlTracker::default()),
            flash_volume,
            financial_manager,
            rent_recovery,
            price_aggregator,
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

        // 1a. Ingest recent trades for PnL tracking
        let last_trade_ts = self
            .database
            .get_state("pnl_last_trade_ts")
            .await?
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0);
        let last_trade_ids = self
            .database
            .get_state("pnl_last_trade_ids")
            .await?
            .and_then(|value| serde_json::from_str::<Vec<String>>(&value).ok())
            .unwrap_or_default();
        let trades_for_pnl = self.database.get_recent_trades(last_trade_ts).await?;

        if !trades_for_pnl.is_empty() {
            let mut tracker = self.pnl_tracker.lock().await;
            let mut newest_ts = last_trade_ts;
            let mut newest_ids = if newest_ts == last_trade_ts {
                last_trade_ids.clone()
            } else {
                Vec::new()
            };
            let mut last_ids_set: HashSet<String> = if newest_ts == last_trade_ts {
                last_trade_ids.into_iter().collect()
            } else {
                HashSet::new()
            };
            let mut processed_any = false;

            for trade in trades_for_pnl {
                if trade.timestamp < last_trade_ts {
                    continue;
                }

                if trade.timestamp == last_trade_ts && last_ids_set.contains(&trade.id) {
                    continue;
                }

                if trade.timestamp > newest_ts {
                    newest_ts = trade.timestamp;
                    newest_ids.clear();
                    last_ids_set.clear();
                }

                tracker.record_trade(trade.side, trade.price, trade.volume);
                processed_any = true;

                if trade.timestamp == newest_ts && last_ids_set.insert(trade.id.clone()) {
                    newest_ids.push(trade.id.clone());
                }
            }

            if processed_any {
                self.database
                    .set_state("pnl_last_trade_ts", &newest_ts.to_string())
                    .await?;
                self.database
                    .set_state("pnl_last_trade_ids", &serde_json::to_string(&newest_ids)?)
                    .await?;
            }
        }

        // 2. Fetch live market data (with fallback for Phase 1 simulation)
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let market_id = &self._settings.openbook_market_id;

        // Try to get real price from aggregator first
        let real_bmv_price = self
            .price_aggregator
            .fetch_price_native(market_id)
            .await
            .ok();

        let market_data = match self.solana.get_market_data(market_id).await {
            Ok(mut data) => {
                if let Some(p) = real_bmv_price {
                    data.price = p; // override with real aggregator price if available
                }
                data
            }
            Err(e) => {
                warn!(error = %e, ?market_id, "Failed to fetch market data from Solana RPC");
                let price = if let Some(p) = real_bmv_price {
                    p
                } else {
                    let fallback = self.pivot_engine.get_last_pivot().await;
                    if fallback.is_zero() {
                        Decimal::from(150)
                    } else {
                        fallback
                    }
                };

                crate::domain::MarketUpdate {
                    price,
                    volume_24h: Decimal::from(1000),
                    timestamp: now,
                }
            }
        };
        // Ensure pivot engine is updated with this price even in simulation
        self.pivot_engine.set_last_price(market_data.price).await;

        // 3. Fetch recent trades from DB for VWAP
        let lookback_secs = self.pivot_engine.lookback_window_seconds();
        let recent_trades = self.database.get_recent_trades(now - lookback_secs).await?;

        // 4. Compute Elapsed seconds for Seeded Pivot
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

        // 5a. Emit PnL Metrics
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

        // 5aa. SOL/USDC and Cross-rate (v0.3.0 Requirement)
        let sol_usdc_id = &self._settings.sol_usdc_market_id;
        let sol_usdc_price = match self.price_aggregator.fetch_price_usd(sol_usdc_id).await {
            Ok(p) => p,
            Err(e) => {
                debug!(error = %e, ?sol_usdc_id, "Aggregator failed for SOL/USDC, trying Solana RPC");
                match self.solana.get_market_data(sol_usdc_id).await {
                    Ok(data) => data.price,
                    Err(rpc_e) => {
                        warn!(error = %rpc_e, "Solana RPC also failed for SOL/USDC, using fallback 150.0");
                        Decimal::from(150)
                    }
                }
            }
        };
        let bmv_price_sol = market_data.price;
        let bmv_price_usdc = bmv_price_sol * sol_usdc_price;

        gauge!("bot_sol_usdc_price", sol_usdc_price.to_f64().unwrap_or(0.0));
        gauge!("bot_bmv_price_sol", bmv_price_sol.to_f64().unwrap_or(0.0));
        gauge!("bot_bmv_price_usdc", bmv_price_usdc.to_f64().unwrap_or(0.0));

        // 5b. Target Control (v0.3.0 Requirement)
        // TARGET_CONTROL_% = 100% − LockedTokens − OwnedTokens
        let total_emission = self._settings.target_control.total_emission;
        let locked_tokens = self._settings.target_control.locked_tokens;
        let owned_tokens = pnl_snapshot.net_position; // Owned by bot (realized + unrealized)

        let free_emission = total_emission - locked_tokens - owned_tokens;
        let target_control_percent = if total_emission.is_zero() {
            Decimal::ZERO
        } else {
            (free_emission / total_emission) * Decimal::from(100)
        };

        gauge!(
            "bot_target_control_percent",
            target_control_percent.to_f64().unwrap_or(0.0)
        );
        gauge!("bot_free_emission", free_emission.to_f64().unwrap_or(0.0));

        info!(
            %target_control_percent,
            %free_emission,
            "Target Control: emission status computed"
        );

        // 6. Check if we need to rebuild the grid
        if self.rebalance_service.should_rebuild(pivot) {
            info!(?pivot, "Rebuilding grid...");

            // 7. Build Grid
            let grid = self.grid_builder.build(pivot, Decimal::from(100)).await;

            gauge!("bot_grid_levels_count", grid.len() as f64);
            info!(levels = grid.len(), "Grid constructed");

            // 8. Execute Grid Update & Emit Metrics
            let mut total_depth = Decimal::ZERO;
            for (idx, level) in grid.iter().enumerate() {
                let side_str = match level.side {
                    crate::domain::OrderSide::Buy => "BUY",
                    crate::domain::OrderSide::Sell => "SELL",
                };

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
            gauge!("bot_fill_rate_percent", 88.0); // Mock
            gauge!("bot_bundle_latency_ms", 42.0); // Mock
        } else {
            info!("Pivot stable, no rebalance needed");
        }

        // 10. Execute Flash Volume cycle
        if let Err(e) = self.flash_volume.execute_cycle().await {
            warn!(error = %e, "flash_volume_cycle_failed");
        }

        // 11. Execute Financial Manager checks
        self.financial_manager.check_balances().await?;
        self.financial_manager
            .rebalance_fiat(market_data.price, pivot)
            .await?;

        // 12. Periodic Rent Recovery
        self.rent_recovery.recover_rent().await?;

        // 13. Save price history for dashboard visualization
        if let Err(e) = self
            .database
            .save_price_tick(bmv_price_sol, sol_usdc_price)
            .await
        {
            error!(error = %e, "failed_to_save_price_tick");
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
                .find_open_orders(market_id, &(*wallet).pubkey())
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
                .cancel_all_orders(market_id, &wallet, jito_url, tip_lamports)
                .await?;
            info!(wallet = %wallet.pubkey(), %result, "Cancel-all submitted");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::mocks::{MockDatabaseProvider, MockSolanaProvider};
    use crate::utils::BotSettings;
    use mockall::predicate::*;
    use rust_decimal_macros::dec;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_trading_service_tick_basic() {
        let mut mock_solana = MockSolanaProvider::new();
        let mut mock_database = MockDatabaseProvider::new();

        let settings = BotSettings::default();

        // Mock get_market_data
        let market_id_clone = settings.openbook_market_id.clone();
        mock_solana
            .expect_get_market_data()
            .with(eq(market_id_clone))
            .returning(|_| {
                Ok(crate::domain::MarketUpdate {
                    price: dec!(100.5),
                    volume_24h: dec!(1000000),
                    timestamp: 123456789,
                })
            });

        // Mock SOL/USDC market data
        let sol_usdc_id_clone = settings.sol_usdc_market_id.clone();
        mock_solana
            .expect_get_market_data()
            .with(eq(sol_usdc_id_clone))
            .returning(|_| {
                Ok(crate::domain::MarketUpdate {
                    price: dec!(150.0),
                    volume_24h: dec!(5000000),
                    timestamp: 123456789,
                })
            });

        // Mock recent trades
        mock_database
            .expect_get_recent_trades()
            .returning(|_| Ok(vec![]));

        // Mock state
        mock_database.expect_get_state().returning(|_| Ok(None));

        // Elaboration for v0.3.0 modules:
        mock_solana
            .expect_get_balance()
            .returning(|_| Ok(1_000_000_000));
        mock_solana
            .expect_get_token_balance()
            .returning(|_, _| Ok(1_000_000));
        mock_solana
            .expect_find_open_orders()
            .returning(|_, _| Ok(None));

        mock_database
            .expect_save_price_tick()
            .returning(|_, _| Ok(()));

        // Settings to disable noisy modules if possible or just handle them
        let mut settings = BotSettings::default();
        settings.flash_volume.enabled = false;

        let solana: Arc<dyn SolanaProvider> = Arc::new(mock_solana);
        let database: Arc<dyn DatabaseProvider> = Arc::new(mock_database);
        let wallet_manager = Arc::new(
            crate::infra::WalletManager::new(&[
                solana_sdk::signature::Keypair::new().to_base58_string()
            ])
            .unwrap(),
        );

        let pivot_engine = Arc::new(PivotEngine::new(
            dec!(100.5),
            7,
            60,
            dec!(1000000),
            dec!(0.02),
            dec!(0.01),
            dec!(0.001),
            dec!(10),
        ));

        let price_aggregator = Arc::new(PriceAggregator::default());
        let service = TradingService::new(
            settings,
            solana,
            database,
            wallet_manager,
            pivot_engine,
            price_aggregator,
        );

        let result = service.tick().await;
        result.expect("Trading service tick failed");
    }
}
