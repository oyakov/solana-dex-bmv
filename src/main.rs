use solana_dex_bmv::infra::{
    Database, HealthChecker, PriceAggregator, SolanaClient, WalletManager,
};
use solana_dex_bmv::services::{MarketDataService, PivotEngine, TradingService};
use solana_dex_bmv::utils::BotSettings;

use anyhow::{Context, Result};
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;
use tracing::{error, info, Level};
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
    solana_dex_bmv::infra::observability::init_metrics();

    info!("Solana DEX BMV bot (Rust) starting");

    // Load settings
    let settings = BotSettings::load()?;

    // Initialize infrastructure
    let commitment = CommitmentConfig::confirmed();
    let solana = Arc::new(SolanaClient::new(
        &settings.rpc_endpoints.primary_http,
        commitment,
    ));
    let database = Arc::new(Database::connect(&settings.database.url).await?);
    let wallet_manager = Arc::new(WalletManager::new(&settings.wallets.multi_wallet.keypairs)?);
    let price_aggregator = Arc::new(PriceAggregator::new());

    // Perform connectivity health checks
    let health_checker = HealthChecker::new(solana.clone(), database.clone(), settings.clone());
    let health_reports = health_checker.run_all_checks().await;
    HealthChecker::display_reports(&health_reports);
    health_checker
        .verify_critical_services(&health_reports)
        .await?;

    // Spawn periodic health check task
    let health_checker_task = Arc::new(health_checker);
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
    let pivot_engine = Arc::new(PivotEngine::new(
        settings.pivot_vwap.pivot_price,
        settings.pivot_vwap.lookback_days,
        settings.pivot_vwap.lookback_minutes,
        settings.pivot_vwap.nominal_daily_volume,
        settings.pivot_vwap.market_id_rent_sol,
        settings.pivot_vwap.account_rent_sol,
        settings.pivot_vwap.jito_tip_sol,
        settings.pivot_vwap.fee_bps,
    ));

    // Initialize and spawn Market Data Service (WebSocket ingestion)
    let market_data_service = MarketDataService::new(
        &settings.rpc_endpoints.primary_ws,
        database.clone(),
        &settings.openbook_market_id,
        pivot_engine.clone(),
    );
    tokio::spawn(async move {
        if let Err(e) = market_data_service.run().await {
            error!(error = ?e, "MarketDataService failed");
        }
    });

    // Initialize and spawn API Server
    let api_server = solana_dex_bmv::infra::ApiServer::new(
        settings.clone(),
        database.clone(),
        solana.clone(),
        wallet_manager.clone(),
        pivot_engine.clone(),
    );
    tokio::spawn(async move {
        if let Err(e) = api_server.run().await {
            error!(error = ?e, "ApiServer failed");
        }
    });

    // Initialize orchestrator
    let orchestrator = TradingService::new(
        settings,
        solana,
        database,
        wallet_manager,
        pivot_engine,
        price_aggregator,
    );

    // Setup signal handling for graceful shutdown
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        let _ = tx.send(()).await;
    });

    info!("Bot is running. Press Ctrl+C to stop.");

    // Run the trading loop with select for signal
    tokio::select! {
        res = orchestrator.run() => {
            if let Err(e) = res {
                error!(error = ?e, "Trading loop failed");
            }
        }
        _ = rx.recv() => {
            info!("Shutdown signal received");
        }
    }

    info!("Solana DEX BMV bot shutting down gracefully");
    Ok(())
}
