use crate::domain::{OrderSide, Trade};
use crate::infra::Database;
use crate::services::PivotEngine;
use crate::utils::BotSettings;
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct IncomingTrade {
    id: String,
    timestamp: i64,
    price: Decimal,
    volume: Decimal,
    side: String,
    wallet: String,
}

#[derive(Debug, Deserialize)]
struct TradeEnvelope {
    trade: IncomingTrade,
}

pub struct TradeStreamService {
    settings: BotSettings,
    database: Arc<Database>,
    pivot_engine: PivotEngine,
}

impl TradeStreamService {
    pub fn new(settings: BotSettings, database: Arc<Database>, pivot_engine: PivotEngine) -> Self {
        Self {
            settings,
            database,
            pivot_engine,
        }
    }

    pub async fn run(&self) -> Result<()> {
        self.seed_cache_from_db().await?;

        let mut backoff = Duration::from_secs(1);
        loop {
            let result = self.connect_and_stream().await;
            if let Err(err) = result {
                warn!(error = ?err, "trade_stream_disconnected_retrying");
            }

            tokio::time::sleep(backoff).await;
            backoff = if result.is_ok() {
                Duration::from_secs(1)
            } else {
                std::cmp::min(backoff * 2, Duration::from_secs(30))
            };
        }
    }

    async fn seed_cache_from_db(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let since_timestamp = now - self.pivot_engine.lookback_window_seconds();
        let trades = self.database.get_recent_trades(since_timestamp).await?;
        info!(count = trades.len(), "seeded_trade_cache_from_db");
        self.pivot_engine.seed_trades(trades).await;
        Ok(())
    }

    async fn connect_and_stream(&self) -> Result<()> {
        let ws_url = &self.settings.rpc_endpoints.primary_ws;
        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_url)
            .await
            .with_context(|| format!("failed_to_connect_ws {ws_url}"))?;
        info!(ws_url, "trade_stream_connected");

        self.seed_cache_from_db().await?;

        let (mut write, mut read) = ws_stream.split();

        let subscription = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "logsSubscribe",
            "params": [
                { "mentions": [self.settings.openbook_market_id] },
                { "commitment": "confirmed" }
            ]
        });
        write
            .send(Message::Text(subscription.to_string()))
            .await
            .context("failed_to_send_trade_subscription")?;

        let mut last_trade_timestamp: Option<i64> = None;

        while let Some(message) = read.next().await {
            match message? {
                Message::Text(text) => {
                    if let Some(trade) = parse_trade_message(&text) {
                        if let Some(last) = last_trade_timestamp {
                            if trade.timestamp > last + 60 {
                                warn!(
                                    previous = last,
                                    current = trade.timestamp,
                                    "trade_gap_detected"
                                );
                            }
                        }

                        last_trade_timestamp = Some(trade.timestamp);
                        self.database.save_trade(&trade).await?;
                        self.pivot_engine.record_trade(trade).await;
                    }
                }
                Message::Ping(payload) => {
                    write.send(Message::Pong(payload)).await?;
                }
                Message::Close(frame) => {
                    warn!(?frame, "trade_stream_closed");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

fn parse_trade_message(text: &str) -> Option<Trade> {
    let trade = if let Ok(wrapper) = serde_json::from_str::<TradeEnvelope>(text) {
        wrapper.trade
    } else if let Ok(trade) = serde_json::from_str::<IncomingTrade>(text) {
        trade
    } else {
        return None;
    };

    let side = match trade.side.to_lowercase().as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => {
            warn!(side = trade.side, "unknown_trade_side");
            return None;
        }
    };

    Some(Trade {
        id: trade.id,
        timestamp: trade.timestamp,
        price: trade.price,
        volume: trade.volume,
        side,
        wallet: trade.wallet,
    })
}
