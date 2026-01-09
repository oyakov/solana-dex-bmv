use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatQuote {

    pub provider: String,
    pub pair: String, // e.g., "USD/SOL"
    pub price: Decimal,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSnapshot {

    pub owner: String,
    pub balance_lamports: u64,
    pub token_balances: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub timestamp: i64,
    pub price: Decimal,
    pub volume: Decimal,
    pub side: OrderSide,
    pub wallet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketUpdate {
    pub timestamp: i64,
    pub price: Decimal,
    pub volume_24h: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookLevel {
    pub price: Decimal,
    pub size: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub market_id: String,
    pub timestamp: i64,
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
}

impl Orderbook {
    pub fn get_mid_price(&self) -> Option<Decimal> {
        let best_bid = self.bids.first()?.price;
        let best_ask = self.asks.first()?.price;
        Some((best_bid + best_ask) / Decimal::from(2))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {

    pub transaction_id: String,
    pub status: String, // "success", "failed", "pending"
    pub error: Option<String>,
    pub timestamp: i64,
}



