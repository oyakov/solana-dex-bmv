use crate::infra::{
    Database, DatabaseProvider, HealthChecker, SolanaClient, SolanaProvider, WalletManager,
};
use crate::services::PivotEngine;
use crate::utils::BotSettings;
use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Clone)]
struct ApiState {
    settings: BotSettings,
    database: Arc<Database>,
    solana: Arc<SolanaClient>,
    wallet_manager: Arc<WalletManager>,
    pivot_engine: Arc<PivotEngine>,
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

impl ApiServer {
    pub fn new(
        settings: BotSettings,
        database: Arc<Database>,
        solana: Arc<SolanaClient>,
        wallet_manager: Arc<WalletManager>,
        pivot_engine: Arc<PivotEngine>,
    ) -> Self {
        Self {
            state: ApiState {
                settings,
                database,
                solana,
                wallet_manager,
                pivot_engine,
            },
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

        let app = Router::new()
            .route("/health", get(handle_health))
            .route("/stats", get(handle_stats))
            .route("/history", get(handle_history))
            .route("/latency", get(handle_latency))
            .route("/wallets", get(handle_list_wallets))
            .route("/wallets/add", post(handle_add_wallet))
            .route("/control", post(handle_control))
            .layer(CorsLayer::permissive())
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
    Json(BotStats {
        pivot_price: state.pivot_engine.get_last_pivot().await,
        buy_channel_width: state.settings.channel_bounds.buy_percent,
        sell_channel_width: state.settings.channel_bounds.sell_percent,
        active_wallets: state.wallet_manager.count().await,
        kill_switch_active: false,
    })
}

async fn handle_list_wallets(State(state): State<ApiState>) -> Json<Vec<WalletInfo>> {
    let wallets = state.wallet_manager.get_all_wallets().await;
    let mut info_list = Vec::new();

    let usdc_mint = Pubkey::from_str(&state.settings.wallets.usdc_wallet_3).unwrap_or_default();

    for wallet in wallets {
        let pubkey = wallet.pubkey();
        let pubkey_str = pubkey.to_string();
        let sol_balance =
            state.solana.get_balance(&pubkey_str).await.unwrap_or(0) as f64 / 1_000_000_000.0;
        let usdc_balance_raw: u64 = state
            .solana
            .get_token_balance(&pubkey, &usdc_mint)
            .await
            .unwrap_or(0);
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
    match state.wallet_manager.add_wallet(&payload.secret).await {
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
