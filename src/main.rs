use solana_dex_bmv::infra::{
    Database, DatabaseProvider, HealthChecker, PriceAggregator, SolanaClient, SolanaProvider,
    WalletManager,
};
use solana_dex_bmv::services::{MarketDataService, PivotEngine, TradingService};
use solana_dex_bmv::utils::BotSettings;

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;
use tracing::{error, info, warn, Level};
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
    let settings = Arc::new(tokio::sync::RwLock::new(BotSettings::load()?));

    // Spawn config watcher
    BotSettings::spawn_config_watcher(settings.clone())?;

    let settings_read = settings.read().await;

    let commitment = CommitmentConfig::confirmed();
    let solana = Arc::new(SolanaClient::new(
        &settings_read.rpc_endpoints.primary_http,
        commitment,
    ));
    let database = Arc::new(Database::connect(&settings_read.database.url).await?);
    let wallet_manager = Arc::new(WalletManager::new(
        &settings_read.wallets.multi_wallet.keypairs,
        Some(database.clone()),
    )?);
    wallet_manager.load_from_db().await?;
    println!(
        "WALLET_MANAGER_LOAD_FINISHED: {} wallets",
        wallet_manager.get_all_wallets().await.len()
    );
    let price_aggregator = Arc::new(PriceAggregator::new());

    // Perform connectivity health checks
    let health_checker =
        HealthChecker::new(solana.clone(), database.clone(), (*settings_read).clone());
    let health_reports = health_checker.run_all_checks().await;
    HealthChecker::display_reports(&health_reports);
    health_checker
        .verify_critical_services(&health_reports)
        .await?;

    // Spawn periodic health check task
    let health_checker_task = Arc::new(health_checker);
    let health_check_interval = settings_read.health_check_interval_seconds;
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

    let pivot_engine = Arc::new(PivotEngine::new(
        settings_read.pivot_vwap.pivot_price,
        settings_read.pivot_vwap.lookback_days,
        settings_read.pivot_vwap.lookback_minutes,
        settings_read.pivot_vwap.nominal_daily_volume,
        settings_read.pivot_vwap.market_id_rent_sol,
        settings_read.pivot_vwap.account_rent_sol,
        settings_read.pivot_vwap.jito_tip_sol,
        settings_read.pivot_vwap.fee_bps,
    ));

    // Initialize and spawn Market Data Service (WebSocket ingestion)
    let market_data_service = MarketDataService::new(
        &settings_read.rpc_endpoints.primary_ws,
        database.clone(),
        &settings_read.openbook_market_id,
        pivot_engine.clone(),
    );
    tokio::spawn(async move {
        if let Err(e) = market_data_service.run().await {
            error!(error = ?e, "MarketDataService failed");
        }
    });

    // Initialize Auth
    let auth_secret = std::env::var("AUTH_SECRET")
        .context("AUTH_SECRET environment variable must be set for security")?;
    let auth = Arc::new(solana_dex_bmv::infra::Auth::new(auth_secret));

    // Initialize and spawn API Server
    let api_server = solana_dex_bmv::infra::ApiServer::new(
        settings.clone(),
        database.clone() as Arc<dyn DatabaseProvider>,
        solana.clone() as Arc<dyn SolanaProvider>,
        wallet_manager.clone(),
        pivot_engine.clone(),
        auth.clone(),
    );

    tokio::spawn(async move {
        if let Err(e) = api_server.run().await {
            error!(error = ?e, "ApiServer failed");
        }
    });

    // Initialize orchestrator
    drop(settings_read);

    let orchestrator = TradingService::new(
        settings.clone(),
        solana,
        database.clone(),
        wallet_manager,
        pivot_engine,
        price_aggregator.clone(),
    )
    .await;

    // Setup signal handling for graceful shutdown
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        let _ = tx.send(()).await;
    });

    info!("Bot is running. Press Ctrl+C to stop.");

    // Perform backfill if requested or needed
    if let Err(e) = backfill_historical_data(database.clone(), price_aggregator.clone()).await {
        warn!(error = ?e, "Historical backfill failed, continuing anyway");
    }

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

async fn backfill_historical_data(
    db: Arc<Database>,
    aggregator: Arc<PriceAggregator>,
) -> Result<()> {
    info!("Checking if price history backfill is needed...");

    // Check if we have recent history (last 1 hour)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let history = db.get_price_history(now - 3600).await?;
    if history.len() > 5 {
        info!("Price history already exists, skipping backfill");
        return Ok(());
    }

    info!("Fetching historical SOL/USDC data from Binance...");
    let sol_history = aggregator.fetch_sol_history(48).await?; // Last 48 hours

    if sol_history.is_empty() {
        warn!("No historical data returned from Binance");
        return Ok(());
    }

    // Use current BMV price for the backfill
    let bmv_price = aggregator
        .fetch_price_native("B9coHrCxYv7xmPfSU7Z5VfugDqdTdZqZTpBGBdazq8AC")
        .await
        .unwrap_or(Decimal::new(11, 6)); // Fallback to 0.000011

    let mut ticks = Vec::new();
    for (ts, sol_price) in sol_history {
        ticks.push((ts, bmv_price, sol_price));
    }

    db.save_historical_price_ticks(ticks).await?;
    info!("Successfully backfilled historical price data");

    Ok(())
}
