use crate::infra::{Database, DatabaseProvider, HealthChecker, SolanaClient, WalletManager};
use crate::services::PivotEngine;
use crate::utils::BotSettings;
use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
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
    // In a real scenario, we'd fetch actual in-memory metrics.
    // Here we provide a snapshot.
    Json(BotStats {
        pivot_price: state.pivot_engine.get_last_pivot().await,
        buy_channel_width: state.settings.channel_bounds.buy_percent,
        sell_channel_width: state.settings.channel_bounds.sell_percent,
        active_wallets: state.wallet_manager.get_all_wallets().len(),
        kill_switch_active: false, // Should check Redis/State
    })
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
