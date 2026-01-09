use crate::domain::{AssetPosition, MarketUpdate, Trade};
use rust_decimal::prelude::*;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct PivotEngine {
    pub target_allocation_usd: Decimal,
    pub lookback_days: u32,
    pub initial_fade_in_days: u32,
}

impl PivotEngine {
    pub fn new(target_allocation_usd: Decimal) -> Self {
        Self {
            target_allocation_usd,
            lookback_days: 365,
            initial_fade_in_days: 30,
        }
    }

    pub async fn compute_pivot(
        &self,
        _positions: &[AssetPosition],
        historical_trades: &[Trade],
        market_update: Option<&MarketUpdate>,
        days_since_start: u32,
    ) -> Decimal {
        if historical_trades.is_empty() && market_update.is_none() {
            warn!("no_data_for_vwap_calculation");
            return self.target_allocation_usd;
        }

        let mut total_value = Decimal::ZERO;
        let mut total_volume = Decimal::ZERO;

        for trade in historical_trades {
            total_value += trade.price * trade.volume;
            total_volume += trade.volume;
        }

        // Include current market data in VWAP if available
        if let Some(update) = market_update {
            // Note: v2.5 says "Pure VWAP" is Σ(price × volume) / Σ volume
            // We include the latest update to ensure hot state is reflected
            total_value += update.price * update.volume_24h;
            total_volume += update.volume_24h;
        }

        let vwap = if total_volume.is_zero() {
            warn!("total_volume_is_zero_in_vwap");
            market_update.map(|m| m.price).unwrap_or(Decimal::ZERO)
        } else {
            total_value / total_volume
        };

        let current_price = market_update.map(|m| m.price).unwrap_or(Decimal::ZERO);

        let pivot = if days_since_start < self.initial_fade_in_days {
            let fade_ratio =
                Decimal::from(days_since_start) / Decimal::from(self.initial_fade_in_days);
            let p = (current_price * (Decimal::ONE - fade_ratio)) + (vwap * fade_ratio);
            info!(fade_in_active = ?fade_ratio, ?p, ?vwap);
            p
        } else {
            info!(fade_in_complete = ?vwap);
            vwap
        };

        info!(?vwap, current = ?current_price, final_pivot = ?pivot, "pivot_computed");
        pivot
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::OrderSide;

    #[test]
    fn test_compute_pivot_vwap() {
        let engine = PivotEngine::new(Decimal::from(100));
        let historical_trades = vec![
            Trade {
                id: "1".to_string(),
                timestamp: 1000,
                price: Decimal::from(100),
                volume: Decimal::from(10),
                side: OrderSide::Buy,
                wallet: "w1".to_string(),
            },
            Trade {
                id: "2".to_string(),
                timestamp: 2000,
                price: Decimal::from(110),
                volume: Decimal::from(10),
                side: OrderSide::Buy,
                wallet: "w1".to_string(),
            },
        ];

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Days since start > initial_fade_in_days (30)
        let pivot = rt.block_on(engine.compute_pivot(&[], &historical_trades, None, 31));

        // VWAP: (100*10 + 110*10) / 20 = 2100 / 20 = 105
        assert_eq!(pivot, Decimal::from(105));
    }

    #[test]
    fn test_compute_pivot_fade_in() {
        let engine = PivotEngine::new(Decimal::from(100));
        let historical_trades = vec![
            Trade {
                id: "1".to_string(),
                timestamp: 1000,
                price: Decimal::from(100),
                volume: Decimal::from(10),
                side: OrderSide::Buy,
                wallet: "w1".to_string(),
            },
        ];
        let market_update = MarketUpdate {
            timestamp: 2000,
            price: Decimal::from(120),
            volume_24h: Decimal::from(10),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Current price = 120
        // VWAP = (100*10 + 120*10) / 20 = 110
        // Halfway through fade-in (15 days out of 30)
        let pivot = rt.block_on(engine.compute_pivot(&[], &historical_trades, Some(&market_update), 15));

        // Ratio = 0.5
        // Pivot = (120 * 0.5) + (110 * 0.5) = 60 + 55 = 115
        assert_eq!(pivot, Decimal::from(115));
    }

    #[test]
    fn test_compute_pivot_empty_data() {
        let target = Decimal::from(1000);
        let engine = PivotEngine::new(target);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pivot = rt.block_on(engine.compute_pivot(&[], &[], None, 0));
        assert_eq!(pivot, target);
    }

    #[test]
    fn test_compute_pivot_zero_volume() {
        let engine = PivotEngine::new(Decimal::from(100));
        let market_update = MarketUpdate {
            timestamp: 1000,
            price: Decimal::from(150),
            volume_24h: Decimal::ZERO,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pivot = rt.block_on(engine.compute_pivot(&[], &[], Some(&market_update), 31));
        
        // Should fallback to market update price (150)
        assert_eq!(pivot, Decimal::from(150));
    }
}

