use crate::infra::{
    Auth, Database, DatabaseProvider, HealthChecker, SolanaClient, SolanaProvider, WalletManager,
};
use crate::services::{GridBuilder, PivotEngine, SimulationEngine};
use crate::utils::BotSettings;
use anyhow::Result;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::http::{header, Method, StatusCode};
use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};
use futures_util::future;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

#[derive(Clone)]
struct ApiState {
    settings: BotSettings,
    database: Arc<dyn DatabaseProvider>,
    solana: Arc<dyn SolanaProvider>,
    wallet_manager: Arc<WalletManager>,
    pivot_engine: Arc<PivotEngine>,
    auth: Arc<Auth>,
    simulation_engine: Arc<SimulationEngine>,
}

pub struct ApiServer {
    state: ApiState,
}

#[derive(Serialize)]
struct BotStats {
    pivot_price: rust_decimal::Decimal,
    buy_channel_width: rust_decimal::Decimal,
    sell_channel_width: rust_decimal::Decimal,
    active_wallets: usize,
    kill_switch_active: bool,
    total_sol_balance: f64,
    total_usdc_balance: f64,
    // Ecosystem & Orderbook Metrics
    spread_bps: f64,
    imbalance_index: f64,
    top_holders_percent: f64,
    safe_haven_index: f64,
    support_50: rust_decimal::Decimal,
    support_90: rust_decimal::Decimal,
    resistance_50: rust_decimal::Decimal,
    resistance_90: rust_decimal::Decimal,
    bids: Vec<crate::domain::OrderbookLevel>,
    asks: Vec<crate::domain::OrderbookLevel>,
}

#[derive(Serialize)]
struct WalletInfo {
    pubkey: String,
    sol_balance: f64,
    usdc_balance: f64,
}

#[derive(Deserialize)]
struct AddWalletRequest {
    secret: String,
}

#[derive(Deserialize)]
struct ControlAction {
    action: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

impl ApiServer {
    pub fn new(
        settings: BotSettings,
        database: Arc<Database>,
        solana: Arc<SolanaClient>,
        wallet_manager: Arc<WalletManager>,
        pivot_engine: Arc<PivotEngine>,
        auth: Arc<Auth>,
    ) -> Self {
        Self {
            state: ApiState {
                settings,
                database,
                solana,
                wallet_manager,
                pivot_engine,
                auth,
                simulation_engine: Arc::new(SimulationEngine::new(GridBuilder::default())),
            },
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".to_string());

        let mut origins = Vec::new();
        for s in allowed_origins.split(',') {
            if let Ok(value) = s.trim().parse() {
                origins.push(value);
            }
        }

        let cors = CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

        let app = Router::new()
            .route("/api/login", post(handle_login))
            .nest(
                "/api",
                Router::new()
                    .route("/stats", get(handle_stats))
                    .route("/history", get(handle_history))
                    .route("/latency", get(handle_latency))
                    .route("/wallets", get(handle_list_wallets))
                    .route("/wallets/add", post(handle_add_wallet))
                    .route("/control", post(handle_control))
                    .route("/simulation", post(handle_simulation))
                    .route_layer(middleware::from_fn_with_state(
                        self.state.auth.clone(),
                        crate::infra::auth_middleware,
                    )),
            )
            .route("/health", get(handle_health))
            .layer(cors)
            .with_state(self.state);

        info!("API Server starting on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn handle_health(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let health_checker = HealthChecker::new(state.solana, state.database, state.settings);
    let reports = health_checker.run_all_checks().await;
    Json(serde_json::to_value(reports).unwrap_or_default())
}

async fn handle_stats(State(state): State<ApiState>) -> Json<BotStats> {
    info!("GET /api/stats - Starting data aggregation");
    let wallets = state.wallet_manager.get_all_wallets().await;
    let usdc_mint = Pubkey::from_str(&state.settings.wallets.usdc_wallet_3).unwrap_or_default();

    let mut total_sol = 0.0;
    let mut total_usdc = 0.0;

    // Reduce timeout to 500ms per wallet to prevent API blocking
    let balance_futures = wallets.iter().map(|wallet| {
        let solana = state.solana.clone();
        let pubkey = wallet.pubkey();
        let pubkey_str = pubkey.to_string();
        let usdc_mint = usdc_mint.clone();
        async move {
            let sol = match tokio::time::timeout(
                std::time::Duration::from_millis(500),
                solana.get_balance(&pubkey_str),
            )
            .await
            {
                Ok(Ok(b)) => b as f64 / 1_000_000_000.0,
                _ => 0.0,
            };
            let usdc = match tokio::time::timeout(
                std::time::Duration::from_millis(500),
                solana.get_token_balance(&pubkey, &usdc_mint),
            )
            .await
            {
                Ok(Ok(b)) => b as f64 / 1_000_000.0,
                _ => 0.0,
            };
            (sol, usdc)
        }
    });

    let results = future::join_all(balance_futures).await;
    for (sol, usdc) in results {
        total_sol += sol;
        total_usdc += usdc;
    }

    // 1. Orderbook Metrics (V1) - with timeout
    let market_id = &state.settings.openbook_market_id;
    let orderbook = match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        state.solana.get_orderbook(market_id),
    )
    .await
    {
        Ok(result) => result.ok(),
        Err(_) => None,
    };

    let mut spread_bps = 0.0;
    let mut imbalance_index = 0.0;
    let mut bids = Vec::new();
    let mut asks = Vec::new();

    let mut support_50 = rust_decimal::Decimal::ZERO;
    let mut support_90 = rust_decimal::Decimal::ZERO;
    let mut resistance_50 = rust_decimal::Decimal::ZERO;
    let mut resistance_90 = rust_decimal::Decimal::ZERO;

    if let Some(ob) = orderbook {
        bids = ob.bids;
        asks = ob.asks;

        if let (Some(best_bid), Some(best_ask)) = (bids.first(), asks.first()) {
            let mid = (best_bid.price + best_ask.price) / rust_decimal::Decimal::from(2);
            if mid > rust_decimal::Decimal::ZERO {
                let spread = best_ask.price - best_bid.price;
                spread_bps = (spread / mid).to_f64().unwrap_or(0.0) * 10000.0;
            }
        }

        let bid_depth: f64 = bids.iter().map(|l| l.size.to_f64().unwrap_or(0.0)).sum();
        let ask_depth: f64 = asks.iter().map(|l| l.size.to_f64().unwrap_or(0.0)).sum();
        if (bid_depth + ask_depth) > 0.0 {
            imbalance_index = (bid_depth - ask_depth) / (bid_depth + ask_depth);
        }

        // Calculate 50% and 90% liquidity levels
        let find_level = |levels: &Vec<crate::domain::OrderbookLevel>,
                          total_depth: f64,
                          target_percent: f64|
         -> rust_decimal::Decimal {
            let target = total_depth * target_percent;
            let mut current = 0.0;
            for level in levels {
                current += level.size.to_f64().unwrap_or(0.0);
                if current >= target {
                    return level.price;
                }
            }
            levels
                .last()
                .map(|l| l.price)
                .unwrap_or(rust_decimal::Decimal::ZERO)
        };

        support_50 = find_level(&bids, bid_depth, 0.5);
        support_90 = find_level(&bids, bid_depth, 0.9);
        resistance_50 = find_level(&asks, ask_depth, 0.5);
        resistance_90 = find_level(&asks, ask_depth, 0.9);
    }

    // 2. Ecosystem Health (SPL Token) - with timeouts
    let token_mint_str = &state.settings.token_mint;
    let token_mint = Pubkey::from_str(token_mint_str).unwrap_or_default();

    let mut top_holders_percent = 0.0;
    let largest_accounts = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        state.solana.get_token_largest_accounts(&token_mint),
    )
    .await
    .ok()
    .and_then(|r| r.ok());
    
    let token_supply = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        state.solana.get_token_supply(&token_mint),
    )
    .await
    .ok()
    .and_then(|r| r.ok());

    if let (Some(accounts), Some(supply)) = (largest_accounts, token_supply) {
        if supply > 0 {
            let top_10_sum: u64 = accounts.iter().take(10).map(|(_, amt)| amt).sum();
            top_holders_percent = (top_10_sum as f64 / supply as f64) * 100.0;
        }
    }

    // 3. Safe Haven Index (Correlation)
    // For now, using a simplified 24h performance delta
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let since = now - 86400;
    let mut safe_haven_index = 1.0;

    if let Ok(history) = state.database.get_price_history(since).await {
        if let (Some(first), Some(last)) = (history.first(), history.last()) {
            let sol_change = if first.sol_price > rust_decimal::Decimal::ZERO {
                (last.sol_price - first.sol_price) / first.sol_price
            } else {
                rust_decimal::Decimal::ZERO
            };
            let bmv_change = if first.asset_price > rust_decimal::Decimal::ZERO {
                (last.asset_price - first.asset_price) / first.asset_price
            } else {
                rust_decimal::Decimal::ZERO
            };

            // If SOL drops and BMV is stable/up, index > 1
            let sol_perf = sol_change.to_f64().unwrap_or(0.0);
            let bmv_perf = bmv_change.to_f64().unwrap_or(0.0);

            if sol_perf < 0.0 {
                safe_haven_index = (1.0 + bmv_perf) / (1.0 + sol_perf);
            } else {
                safe_haven_index = (1.0 + bmv_perf) / (1.0 + sol_perf);
            }
        }
    }

    let res = Json(BotStats {
        pivot_price: state.pivot_engine.get_last_pivot().await,
        buy_channel_width: state.settings.channel_bounds.buy_percent,
        sell_channel_width: state.settings.channel_bounds.sell_percent,
        active_wallets: wallets.len(),
        kill_switch_active: false,
        total_sol_balance: total_sol,
        total_usdc_balance: total_usdc,
        spread_bps,
        imbalance_index,
        top_holders_percent,
        safe_haven_index,
        support_50,
        support_90,
        resistance_50,
        resistance_90,
        bids: bids.into_iter().take(20).collect(), // Send top 20 for depth chart
        asks: asks.into_iter().take(20).collect(),
    });
    info!("GET /api/stats - Aggregation complete");
    res
}

async fn handle_list_wallets(State(state): State<ApiState>) -> Json<Vec<WalletInfo>> {
    info!("GET /api/wallets - Listing wallets");
    let wallets = state.wallet_manager.get_all_wallets().await;
    let mut info_list = Vec::new();

    let usdc_mint = Pubkey::from_str(&state.settings.wallets.usdc_wallet_3).unwrap_or_default();

    for wallet in wallets {
        let pubkey = wallet.pubkey();
        let pubkey_str = pubkey.to_string();
        let sol_balance = match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            state.solana.get_balance(&pubkey_str),
        )
        .await
        {
            Ok(Ok(b)) => b as f64 / 1_000_000_000.0,
            _ => 0.0,
        };
        let usdc_balance_raw: u64 = match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            state.solana.get_token_balance(&pubkey, &usdc_mint),
        )
        .await
        {
            Ok(Ok(b)) => b,
            _ => 0,
        };
        let usdc_balance = usdc_balance_raw as f64 / 1_000_000.0;

        info_list.push(WalletInfo {
            pubkey: pubkey_str,
            sol_balance,
            usdc_balance,
        });
    }

    Json(info_list)
}

async fn handle_add_wallet(
    State(state): State<ApiState>,
    Json(payload): Json<AddWalletRequest>,
) -> Json<serde_json::Value> {
    match state.wallet_manager.add_wallet(&payload.secret, true).await {
        Ok(pubkey) => Json(serde_json::json!({
            "status": "ok",
            "pubkey": pubkey
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

async fn handle_history(State(state): State<ApiState>) -> Json<Vec<crate::domain::PriceTick>> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let since = now - 24 * 3600; // Last 24 hours
    let history = state
        .database
        .get_price_history(since)
        .await
        .unwrap_or_default();
    Json(history)
}

async fn handle_latency(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let since = now - 24 * 3600;

    let services = [
        "Solana RPC",
        "Database (Postgres)",
        "Database (PostgreSQL)",
        "Jito Bundler",
        "OpenBook DEX",
    ];

    let mut all_history = std::collections::HashMap::new();

    for service in services {
        let history = state
            .database
            .get_latency_history(service, since)
            .await
            .unwrap_or_default();
        if !history.is_empty() {
            all_history.insert(service, history);
        }
    }

    Json(serde_json::to_value(all_history).unwrap_or_default())
}

async fn handle_control(
    State(_state): State<ApiState>,
    Json(payload): Json<ControlAction>,
) -> Json<serde_json::Value> {
    info!(action = %payload.action, "Received control action");

    match payload.action.as_str() {
        "kill_switch" => {
            // TODO: Implement actual trigger
            Json(serde_json::json!({"status": "ok", "message": "Kill switch toggled"}))
        }
        "rebalance" => Json(serde_json::json!({"status": "ok", "message": "Rebalance triggered"})),
        _ => Json(serde_json::json!({"status": "error", "message": "Unknown action"})),
    }
}

async fn handle_login(
    State(state): State<ApiState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let dashboard_password_hash = std::env::var("DASHBOARD_PASSWORD_HASH").map_err(|_| {
        error!("DASHBOARD_PASSWORD_HASH environment variable must be set");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let parsed_hash = PasswordHash::new(&dashboard_password_hash).map_err(|e| {
        error!("Invalid password hash format: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let token = state.auth.generate_token("admin").map_err(|e| {
            error!("Token generation failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        Ok(Json(LoginResponse { token }))
    } else {
        warn!(
            "Unauthorized login attempt with password length {}",
            payload.password.len()
        );
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Deserialize)]
struct SimulationRequest {
    scenario: crate::services::ScenarioType,
    base_price: rust_decimal::Decimal,
    steps: usize,
    volatility: rust_decimal::Decimal,
}

async fn handle_simulation(
    State(state): State<ApiState>,
    Json(payload): Json<SimulationRequest>,
) -> Json<crate::services::SimulationResult> {
    info!(scenario = ?payload.scenario, "Running simulation");
    let result = state
        .simulation_engine
        .run_simulation(
            payload.scenario,
            payload.base_price,
            payload.steps,
            payload.volatility,
        )
        .await;
    Json(result)
}

use tracing::warn;
