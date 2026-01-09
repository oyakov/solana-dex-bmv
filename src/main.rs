mod domain;
mod infra;
mod services;
mod utils;

use crate::infra::{Database, SolanaClient, WalletManager};
use crate::services::{GridBuilder, PivotEngine, TradingService};
use crate::utils::BotSettings;
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("setting default subscriber failed")?;

    info!("Solana DEX BMV bot (Rust) starting");

    // Load settings
    let settings = BotSettings::load()?;

    // Initialize infrastructure
    let commitment = CommitmentConfig::from_str(&settings.solana.commitment)
        .unwrap_or(CommitmentConfig::confirmed());
    let solana = SolanaClient::new(&settings.solana.rpc_url, commitment);

    let database = Database::connect(&settings.database.path).await?;
    let wallet_manager = WalletManager::new(&settings.solana.wallets)?;

    // Initialize logic services
    let pivot_engine = PivotEngine::new(Decimal::from(1000));
    let grid_builder = GridBuilder {
        orders_per_side: settings.strategy.orders_per_side,
        buy_channel_width: settings.strategy.buy_channel_width,
        sell_channel_width: settings.strategy.sell_channel_width,
    };

    // Initialize orchestrator
    let orchestrator = TradingService::new(
        settings.clone(),
        solana,
        database,
        wallet_manager,
        pivot_engine,
        grid_builder,
    );

    // Run the trading loop
    // Note: This will run forever until interrupted
    orchestrator.run().await?;

    info!("Solana DEX BMV bot shutting down gracefully");
    Ok(())
}
