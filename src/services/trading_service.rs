use crate::infra::{Database, SolanaClient, WalletManager};
use crate::services::{GridBuilder, PivotEngine};
use crate::utils::BotSettings;
use crate::domain::AssetPosition;
use anyhow::Result;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{info, error};


pub struct TradingService {
    _settings: BotSettings,
    solana: SolanaClient,
    database: Database,
    wallet_manager: WalletManager,
    pivot_engine: PivotEngine,
    grid_builder: GridBuilder,
}

impl TradingService {
    pub fn new(
        settings: BotSettings,
        solana: SolanaClient,
        database: Database,
        wallet_manager: WalletManager,
        pivot_engine: PivotEngine,
        grid_builder: GridBuilder,
    ) -> Self {
        Self {
            _settings: settings,
            solana,
            database,
            wallet_manager,
            pivot_engine,
            grid_builder,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            wallet_count = self.wallet_manager.count(),
            "Starting main trading loop"
        );

        let mut iterations = 0;
        loop {
            iterations += 1;
            info!(?iterations, "Trading loop tick");

            if let Err(e) = self.tick().await {
                error!(error = ?e, "Error during trading tick");
            }

            // In a real bot, interval would come from settings
            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn tick(&self) -> Result<()> {
        // 1. Fetch current portfolio and market data for each wallet
        for i in 0..self.wallet_manager.count() {
            let pubkey = self.wallet_manager.get_keypair(i)?.pubkey().to_string();
            let balance = self.solana.get_balance(&pubkey).await?;
            info!(wallet = %pubkey, ?balance, "Checked wallet balance");
        }

        let positions = vec![AssetPosition {
            symbol: "SOL".to_string(),
            quantity: Decimal::from(2),
            notional_usd: Decimal::from(300),
        }];

        // Fetch historical trades from last 365 days (TRS v2.5 requirement)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;
        let lookback = 365 * 24 * 60 * 60;
        let historical_trades = self.database.get_recent_trades(now - lookback).await?;

        // Fetch real market data (Phase 1 Market Data Pipeline)
        let market_update = self.solana
            .get_market_data(&self._settings.strategy.market_id)
            .await?;

        // 2. Re-calculate pivot using historical data + latest update
        let pivot = self.pivot_engine
            .compute_pivot(&positions, &historical_trades, Some(&market_update), 0)
            .await;

        self.database.set_state("pivot", &pivot.to_string()).await?;
        info!(?pivot, history_count = historical_trades.len(), "Pivot updated in database");



        // 3. Build grid
        let grid = self.grid_builder
            .build(pivot, Decimal::from(1)) // size 1 SOL for now
            .await;

        info!(levels = grid.len(), "Grid levels generated");

        // 4. (Planned) Sync orders with the DEX
        // This is where we'll compare current orders on-chain with the generated grid
        // and send bundles to Jito.

        Ok(())
    }
}
