use crate::infra::{Database, SolanaClient, WalletManager};
use crate::services::{GridBuilder, PivotEngine, RebalanceService};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use tracing::{info, error};
use metrics::{counter, gauge};

pub struct TradingService {
    _settings: BotSettings,
    solana: std::sync::Arc<SolanaClient>,
    #[allow(dead_code)]
    database: std::sync::Arc<Database>,
    wallet_manager: std::sync::Arc<WalletManager>,

    pivot_engine: PivotEngine,
    grid_builder: GridBuilder,
    rebalance_service: RebalanceService,
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
            orders_per_side: settings.strategy.orders_per_side,
            buy_channel_width: settings.strategy.buy_channel_width,
            sell_channel_width: settings.strategy.sell_channel_width,
        };

        let rebalance_service = RebalanceService::new(
            solana.clone(),
            wallet_manager.clone(),
            settings.clone(),
        );

        Self {
            _settings: settings,
            solana,
            database,
            wallet_manager,
            pivot_engine,
            grid_builder,
            rebalance_service,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            wallet_count = self.wallet_manager.count(),
            "Starting TradingService main loop"
        );

        loop {
            counter!("bot_ticks_total", 1);
            if let Err(e) = self.tick().await {
                counter!("bot_tick_errors_total", 1);
                error!(error = ?e, "Error during trading tick");
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    }

    pub async fn tick(&self) -> Result<()> {
        info!("Trading tick starting");

        // 1. Rebalance wallets if needed
        self.rebalance_service.rebalance().await?;

        // 2. Fetch live market data (now from L2 orderbook)
        let market_id = &self._settings.strategy.market_id;
        let market_data = self.solana.get_market_data(market_id).await?;

        // 3. Compute Pivot (Pass empty history and 31 days for now)
        let pivot = self.pivot_engine.compute_pivot(&[], &[], Some(&market_data), 31).await;
        gauge!("bot_last_pivot_price", pivot.to_f64().unwrap_or(0.0));
        info!(?pivot, "New pivot calculated");

        // 4. Build Grid
        let grid = self.grid_builder.build(market_data.price, Decimal::from(100)).await;
        gauge!("bot_grid_levels_count", grid.len() as f64);
        info!(levels = grid.len(), "Grid constructed");

        // 5. Execute Grid (Phase 2 core)
        for level in grid {
            let side = match level.side {
                crate::domain::OrderSide::Buy => 0u8,
                crate::domain::OrderSide::Sell => 1u8,
            };
            
            if let Some(_wallet) = self.wallet_manager.get_keypair(0).ok() {

                let price_u64 = (level.price * Decimal::from(1_000_000)).to_u64().unwrap_or(0);
                let size_u64 = (level.size * Decimal::from(1_000_000_000)).to_u64().unwrap_or(0);
                
                info!(?side, ?price_u64, ?size_u64, "Placing grid order (Phase 2 execution)");
                // self.solana.place_order(market_id, *wallet, side, price_u64, size_u64, ...).await?;
            }
        }

        Ok(())
    }
}

