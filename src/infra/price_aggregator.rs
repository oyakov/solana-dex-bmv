use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{debug, warn};

#[derive(Debug, Deserialize)]
struct DexScreenerResponse {
    pairs: Option<Vec<DexPair>>,
}

#[derive(Debug, Deserialize)]
struct DexPair {
    #[serde(rename = "priceUsd")]
    price_usd: Option<String>,
    #[serde(rename = "priceNative")]
    price_native: Option<String>,
}

pub struct PriceAggregator {
    client: reqwest::Client,
}

impl PriceAggregator {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_price_usd(&self, pair_address: &str) -> Result<Decimal> {
        let url = format!(
            "https://api.dexscreener.com/latest/dex/pairs/solana/{}",
            pair_address
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<DexScreenerResponse>()
            .await?;

        let price_str = resp
            .pairs
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.price_usd)
            .ok_or_else(|| anyhow!("No price data found for pair {}", pair_address))?;

        Decimal::from_str(&price_str)
            .map_err(|e| anyhow!("Failed to parse price '{}': {}", price_str, e))
    }

    pub async fn fetch_price_native(&self, pair_address: &str) -> Result<Decimal> {
        let url = format!(
            "https://api.dexscreener.com/latest/dex/pairs/solana/{}",
            pair_address
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<DexScreenerResponse>()
            .await?;

        let price_str = resp
            .pairs
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.price_native)
            .ok_or_else(|| anyhow!("No native price data found for pair {}", pair_address))?;

        Decimal::from_str(&price_str)
            .map_err(|e| anyhow!("Failed to parse native price '{}': {}", price_str, e))
    }
}
