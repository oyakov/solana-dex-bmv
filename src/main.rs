mod domain;
mod infra;
mod services;
mod utils;

use crate::infra::{Database, SolanaClient, WalletManager};
use crate::services::{PivotEngine, TradingService};

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

    // Initialize metrics
    infra::observability::init_metrics();

    info!("Solana DEX BMV bot (Rust) starting");

    // Load settings
    let settings = BotSettings::load()?;

    // Initialize infrastructure
    let commitment = CommitmentConfig::from_str(&settings.solana.commitment)
        .unwrap_or(CommitmentConfig::confirmed());
    let solana = std::sync::Arc::new(SolanaClient::new(&settings.solana.rpc_url, commitment));
    let database = std::sync::Arc::new(Database::connect(&settings.database.path).await?);
    let wallet_manager = std::sync::Arc::new(WalletManager::new(&settings.solana.wallets)?);

    // Initialize logic services
    let pivot_engine = PivotEngine::new(Decimal::from(1000));

    // Initialize orchestrator
    let orchestrator = TradingService::new(
        settings,
        solana,
        database,
        wallet_manager,
        pivot_engine,
    );



    // Run the trading loop
    // Note: This will run forever until interrupted
    orchestrator.run().await?;

    info!("Solana DEX BMV bot shutting down gracefully");
    Ok(())
}
