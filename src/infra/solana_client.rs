use crate::domain::MarketUpdate;
use rust_decimal::Decimal;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use tracing::{info, error};

pub struct SolanaClient {
    client: RpcClient,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, commitment: CommitmentConfig) -> Self {
        Self {
            client: RpcClient::new_with_commitment(rpc_url.to_string(), commitment),
        }
    }

    pub async fn get_balance(&self, owner: &str) -> Result<u64> {
        let pubkey = Pubkey::from_str(owner).map_err(|e| anyhow!("Invalid pubkey: {}", e))?;
        let balance = self.client.get_balance(&pubkey).await?;
        Ok(balance)
    }

    pub async fn get_market_data(&self, _market_id: &str) -> Result<MarketUpdate> {
        // TODO: Implement real OpenBook/Raydium market data fetching.
        // This will involve fetching the market accounts and parsing their state.
        
        // For Phase 1 progress, we simulate the fetch but use real timestamp.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;
            
        Ok(MarketUpdate {
            timestamp: now,
            price: Decimal::from(150), // Simulated real price
            volume_24h: Decimal::from(5000),
        })
    }

    pub async fn health(&self) -> bool {
        match self.client.get_version().await {
            Ok(_) => true,
            Err(e) => {
                error!(error = ?e, "health_check_failed");
                false
            }
        }
    }

    pub async fn send_bundle(&self, transactions: Vec<String>, jito_api_url: &str) -> Result<String> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [transactions]
        });

        info!(count = transactions.len(), api = jito_api_url, "sending_jito_bundle");
        
        let client = reqwest::Client::new();
        let response = client.post(jito_api_url).json(&payload).send().await?;
        let result: serde_json::Value = response.json().await?;

        if let Some(error) = result.get("error") {
            error!(?error, "jito_bundle_error");
            return Err(anyhow!("Jito error: {:?}", error));
        }

        let bundle_id = result["result"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing bundle ID in response"))?
            .to_string();

        info!(?bundle_id, "jito_bundle_sent");
        Ok(bundle_id)
    }
}

