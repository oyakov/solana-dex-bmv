use crate::domain::{OrderSide, Trade};
use crate::infra::database::Database;
use crate::infra::openbook::OPENBOOK_V2_PROGRAM_ID;
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
}

impl MarketDataService {
    pub fn new(ws_url: &str, database: Arc<Database>, market_id: &str) -> Self {
        Self {
            ws_url: ws_url.to_string(),
            database,
            market_id: market_id.to_string(),
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
                        // Extract trade data from log
                        // Example log: "Program log: FillEvent { ... }"
                        // Note: Real implementation would use an event parser or Anchor client.
                        // For this environment, we'll use a simplified regex or string parsing if possible.
                        self.parse_and_save_event(log_str, signature).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn parse_and_save_event(&self, log: &str, signature: &str) -> Result<()> {
        // Mock parsing for now as actual parsing requires a specific schema/IDL decoder
        // In a real scenario, we'd use 'anchor_lang::Event' or similar.

        // Assuming we find a FillEvent in the log
        // log might look like: "Program log: FillEvent { price: 154233, quantity: 1000000, ... }"

        // We'll simulate a trade for now to show the flow
        // In production, this would be accurately parsed.
        if log.contains("FillEvent") {
            let trade = Trade {
                id: format!("{}-{}", signature, 0),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs() as i64,
                price: Decimal::from_str("154.23").unwrap(), // Mock
                volume: Decimal::from_str("1.5").unwrap(),   // Mock
                side: OrderSide::Buy,                        // Mock
                wallet: "unknown".to_string(),
            };

            self.database.save_trade(&trade).await?;
            info!(price = %trade.price, volume = %trade.volume, "trade_ingested");
        }

        Ok(())
    }
}
