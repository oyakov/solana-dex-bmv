mod domain;
mod infra;
mod services;
mod utils;

use crate::domain::AssetPosition;
use crate::infra::{Database, SolanaClient};
use crate::services::{GridBuilder, PivotEngine};
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

    // Initialize services
    let pivot_engine = PivotEngine::new(Decimal::from(1000));
    let grid_builder = GridBuilder {
        orders_per_side: settings.strategy.orders_per_side,
        buy_channel_width: settings.strategy.buy_channel_width,
        sell_channel_width: settings.strategy.sell_channel_width,
    };

    // Simulated execution flow matches Python's run_once
    info!("Performing initial setup and checks");

    let owner = settings
        .solana
        .default_fee_payer
        .clone()
        .unwrap_or_else(|| "11111111111111111111111111111111".to_string());

    // Check balance
    let balance = solana.get_balance(&owner).await?;
    info!(?owner, ?balance, "Owner balance checked");

    // Simulation: Compute pivot
    let positions = vec![AssetPosition {
        symbol: "SOL".to_string(),
        quantity: Decimal::from(2),
        notional_usd: Decimal::from(300),
    }];

    let market_data = vec![(Decimal::from(155), Decimal::from(1000))];
    let pivot = pivot_engine
        .compute_pivot(&positions, &market_data, 0)
        .await;

    database.set_state("pivot", &pivot.to_string()).await?;
    info!(?pivot, "Pivot state saved to database");

    // Simulation: Build grid
    let grid = grid_builder
        .build(
            Decimal::from(150),
            Decimal::from_str_radix("0.1", 10).unwrap(),
        )
        .await;
    info!(levels = grid.len(), "Grid constructed");

    // Shutdown
    database.close().await;
    info!("Solana DEX BMV bot shutting down gracefully");

    Ok(())
}
