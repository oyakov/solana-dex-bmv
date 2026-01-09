use crate::domain::AssetPosition;
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
        market_data: &[(Decimal, Decimal)], // (price, volume)
        days_since_start: u32,
    ) -> Decimal {
        if market_data.is_empty() {
            warn!("no_market_data_for_vwap");
            return self.target_allocation_usd;
        }

        let mut total_value = Decimal::ZERO;
        let mut total_volume = Decimal::ZERO;

        for (price, volume) in market_data {
            total_value += price * volume;
            total_volume += volume;
        }

        let vwap = if total_volume.is_zero() {
            warn!("total_volume_is_zero");
            market_data.last().map(|(p, _)| *p).unwrap_or(Decimal::ZERO)
        } else {
            total_value / total_volume
        };

        let current_price = market_data.last().map(|(p, _)| *p).unwrap_or(Decimal::ZERO);

        let pivot = if days_since_start < self.initial_fade_in_days {
            let fade_ratio =
                Decimal::from(days_since_start) / Decimal::from(self.initial_fade_in_days);
            let p = (current_price * (Decimal::ONE - fade_ratio)) + (vwap * fade_ratio);
            info!(fade_in_active = ?fade_ratio, ?pivot, ?vwap);
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

    #[test]
    fn test_compute_pivot_vwap() {
        let engine = PivotEngine::new(Decimal::from(100));
        let market_data = vec![
            (Decimal::from(100), Decimal::from(10)),
            (Decimal::from(110), Decimal::from(10)),
        ];

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Days since start > initial_fade_in_days (30)
        let pivot = rt.block_on(engine.compute_pivot(&[], &market_data, 31));

        // VWAP: (100*10 + 110*10) / 20 = 2100 / 20 = 105
        assert_eq!(pivot, Decimal::from(105));
    }

    #[test]
    fn test_compute_pivot_fade_in() {
        let engine = PivotEngine::new(Decimal::from(100));
        let market_data = vec![
            (Decimal::from(100), Decimal::from(10)),
            (Decimal::from(120), Decimal::from(10)),
        ];

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Current price = 120
        // VWAP = (100*10 + 120*10) / 20 = 110
        // Halfway through fade-in (15 days out of 30)
        let pivot = rt.block_on(engine.compute_pivot(&[], &market_data, 15));

        // Ratio = 0.5
        // Pivot = (120 * 0.5) + (110 * 0.5) = 60 + 55 = 115
        assert_eq!(pivot, Decimal::from(115));
    }
}
