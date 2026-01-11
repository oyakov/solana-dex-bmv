mod domain;
mod infra;
mod services;
mod utils;

use crate::infra::{Database, SolanaClient, WalletManager};
use crate::services::{PivotEngine, TradingService};

use crate::utils::BotSettings;
use anyhow::{Context, Result};
use solana_sdk::commitment_config::CommitmentConfig;
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
    let commitment = CommitmentConfig::confirmed();
    let solana = std::sync::Arc::new(SolanaClient::new(
        &settings.rpc_endpoints.primary_http,
        commitment,
    ));
    let database = std::sync::Arc::new(Database::connect(&settings.database.url).await?);
    let wallet_manager =
        std::sync::Arc::new(WalletManager::new(&settings.wallets.multi_wallet.keypairs)?);

    // Perform connectivity health checks
    let health_checker =
        infra::HealthChecker::new(solana.clone(), database.clone(), settings.clone());
    let health_reports = health_checker.run_all_checks().await;
    infra::HealthChecker::display_reports(&health_reports);
    health_checker
        .verify_critical_services(&health_reports)
        .await?;

    // Spawn periodic health check task
    let health_checker_task = std::sync::Arc::new(health_checker);
    let health_check_interval = settings.health_check_interval_seconds;
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(health_check_interval));
        // Skip the first tick since we already ran it
        interval.tick().await;
        loop {
            interval.tick().await;
            health_checker_task.run_all_checks().await;
        }
    });

    // Initialize logic services
    let pivot_engine = PivotEngine::new(
        settings.pivot_vwap.pivot_price,
        settings.pivot_vwap.lookback_days,
        settings.pivot_vwap.nominal_daily_volume,
    );

    // Initialize orchestrator
    let orchestrator =
        TradingService::new(settings, solana, database, wallet_manager, pivot_engine);

    // Run the trading loop
    // Note: This will run forever until interrupted
    orchestrator.run().await?;

    info!("Solana DEX BMV bot shutting down gracefully");
    Ok(())
}
