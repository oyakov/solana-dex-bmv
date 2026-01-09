use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Open,
    Filled,
    Canceled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPosition {
    pub symbol: String,
    pub quantity: Decimal,
    pub notional_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLevel {
    pub price: Decimal,
    pub size: Decimal,
    pub side: OrderSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_notional_usd: Decimal,
    pub max_open_orders: u32,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_notional_usd: Decimal::from(1000),
            max_open_orders: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatQuote {
    pub provider: String,
    pub pair: String, // e.g., "USD/SOL"
    pub price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSnapshot {
    pub owner: String,
    pub balance_lamports: u64,
    pub token_balances: HashMap<String, u64>,
}
