use crate::domain::{OrderSide, Trade};
use crate::infra::database::Database;
use crate::infra::openbook::OPENBOOK_V2_PROGRAM_ID;
use crate::services::PivotEngine;
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

pub struct MarketDataService {
    ws_url: String,
    database: Arc<Database>,
    market_id: String,
    pivot_engine: Arc<PivotEngine>,
}

impl MarketDataService {
    pub fn new(
        ws_url: &str,
        database: Arc<Database>,
        market_id: &str,
        pivot_engine: Arc<PivotEngine>,
    ) -> Self {
        Self {
            ws_url: ws_url.to_string(),
            database,
            market_id: market_id.to_string(),
            pivot_engine,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(url = %self.ws_url, market = %self.market_id, "starting_market_data_service");

        let (mut ws_stream, _) = connect_async(&self.ws_url).await?;

        // Subscribe to program logs for OpenBook V2
        let sub_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "logsSubscribe",
            "params": [
                {
                    "mentions": [OPENBOOK_V2_PROGRAM_ID]
                },
                {
                    "commitment": "processed"
                }
            ]
        });

        ws_stream
            .send(Message::Text(sub_request.to_string()))
            .await?;

        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_message(&text).await {
                        warn!(error = %e, "failed_to_handle_ws_message");
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("websocket_connection_closed");
                    break;
                }
                Err(e) => {
                    error!(error = %e, "websocket_error");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn handle_message(&self, text: &str) -> Result<()> {
        let v: Value = serde_json::from_str(text)?;

        // Check if it's a notification
        if v["method"] == "logsNotification" {
            let logs = v["params"]["result"]["value"]["logs"]
                .as_array()
                .ok_or_else(|| anyhow!("Missing logs"))?;
            let signature = v["params"]["result"]["value"]["signature"]
                .as_str()
                .unwrap_or("unknown");

            for log in logs {
                if let Some(log_str) = log.as_str() {
                    if log_str.contains("FillEvent") || log_str.contains("TradeEvent") {
                        self.parse_and_save_event(log_str, signature).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn parse_and_save_event(&self, log: &str, signature: &str) -> Result<()> {
        if log.contains("FillEvent") {
            let trade = Trade {
                id: format!("{}-{}", signature, 0),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs() as i64,
                price: Decimal::from_str("154.23").unwrap(), // Mock parsing
                volume: Decimal::from_str("1.5").unwrap(),   // Mock parsing
                side: OrderSide::Buy,                        // Mock parsing
                wallet: "unknown".to_string(),
            };

            self.database.save_trade(&trade).await?;
            self.pivot_engine.record_trade(trade.clone()).await;
            info!(price = %trade.price, volume = %trade.volume, "trade_ingested_and_cached");
        }

        Ok(())
    }
}
