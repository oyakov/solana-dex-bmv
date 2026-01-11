use crate::domain::{AssetPosition, MarketUpdate, Trade};
use rust_decimal::prelude::*;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct PivotEngine {
    pub seed_price: Decimal,
    pub lookback_minutes: u32,
    pub lookback_days: u32,
    pub nominal_daily_volume: Decimal,
    pub market_id_rent_sol: Decimal,
    pub account_rent_sol: Decimal,
    pub jito_tip_sol: Decimal,
    pub fee_bps: Decimal,
    trade_cache: std::sync::Arc<RwLock<VecDeque<Trade>>>,
}

impl PivotEngine {
    pub fn new(
        seed_price: Decimal,
        lookback_days: u32,
        lookback_minutes: u32,
        nominal_daily_volume: Decimal,
        market_id_rent_sol: Decimal,
        account_rent_sol: Decimal,
        jito_tip_sol: Decimal,
        fee_bps: Decimal,
    ) -> Self {
        Self {
            seed_price,
            lookback_minutes,
            lookback_days,
            nominal_daily_volume,
            market_id_rent_sol,
            account_rent_sol,
            jito_tip_sol,
            fee_bps,
            trade_cache: std::sync::Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub fn lookback_window_seconds(&self) -> i64 {
        let minutes = if self.lookback_minutes > 0 {
            self.lookback_minutes as i64
        } else {
            self.lookback_days as i64 * 24 * 60
        };
        minutes * 60
    }

    pub async fn seed_trades(&self, trades: Vec<Trade>) {
        let mut cache = self.trade_cache.write().await;
        cache.clear();
        cache.extend(trades);
        self.prune_cache_locked(&mut cache);
    }

    pub async fn record_trade(&self, trade: Trade) {
        let mut cache = self.trade_cache.write().await;
        cache.push_back(trade);
        self.prune_cache_locked(&mut cache);
    }

    pub async fn cached_trades(&self) -> Vec<Trade> {
        let mut cache = self.trade_cache.write().await;
        self.prune_cache_locked(&mut cache);
        cache.iter().cloned().collect()
    }

    fn prune_cache_locked(&self, cache: &mut VecDeque<Trade>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let cutoff = now - self.lookback_window_seconds();
        while let Some(front) = cache.front() {
            if front.timestamp < cutoff {
                cache.pop_front();
            } else {
                break;
            }
        }
    }

    pub async fn compute_pivot(
        &self,
        _positions: &[AssetPosition],
        historical_trades: &[Trade],
        market_update: Option<&MarketUpdate>,
        days_since_start: u32,
    ) -> Decimal {
        let mut total_value = Decimal::ZERO;
        let mut total_volume = Decimal::ZERO;

        // 1. Calculate from historical trades
        for trade in historical_trades {
            total_value += trade.price * trade.volume;
            total_volume += trade.volume;
        }

        // 2. Include current market data (last 24h) if available
        if let Some(update) = market_update {
            total_value += update.price * update.volume_24h;
            total_volume += update.volume_24h;
        }

        // 3. Add Seeded Pivot weight
        // If we have less than lookback_days of real data, fill the rest with seed_price
        if days_since_start < self.lookback_days {
            let seed_price = if self.seed_price.is_zero() {
                // If seed_price is not set, use current market price or fallback to 100
                market_update.map(|m| m.price).unwrap_or(Decimal::from(100))
            } else {
                self.seed_price
            };

            let remaining_days = self.lookback_days - days_since_start.min(self.lookback_days);
            let seed_volume = Decimal::from(remaining_days) * self.nominal_daily_volume;

            total_value += seed_price * seed_volume;
            total_volume += seed_volume;

            info!(
                ?days_since_start,
                ?remaining_days,
                ?seed_price,
                ?seed_volume,
                "seeded_pivot_active"
            );
        }

        let cost_sol = self.market_id_rent_sol + self.account_rent_sol + self.jito_tip_sol;
        let reference_price = if !total_volume.is_zero() {
            total_value / total_volume
        } else if let Some(update) = market_update {
            update.price
        } else if !self.seed_price.is_zero() {
            self.seed_price
        } else {
            Decimal::ZERO
        };

        let cost_value = cost_sol * reference_price;
        let fee_rate = self.fee_bps / Decimal::from(10_000);
        let fee_volume = total_volume * fee_rate;
        let adjusted_volume = total_volume - fee_volume;
        let adjusted_value = total_value + cost_value;

        let pivot = if adjusted_volume <= Decimal::ZERO {
            warn!("no_volume_detected_falling_back_to_seed_or_market");
            if !self.seed_price.is_zero() {
                self.seed_price
            } else {
                market_update.map(|m| m.price).unwrap_or(Decimal::ZERO)
            }
        } else {
            adjusted_value / adjusted_volume
        };

        let current_price = market_update.map(|m| m.price).unwrap_or(Decimal::ZERO);
        info!(
            ?pivot,
            current = ?current_price,
            ?total_volume,
            ?fee_volume,
            ?cost_value,
            "pivot_computed"
        );
        pivot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::OrderSide;

    #[test]
    fn test_compute_pivot_vwap() {
        // Seed price 0, but days_since_start 366 means no seed weight
        let engine = PivotEngine::new(
            Decimal::ZERO,
            365,
            0,
            Decimal::from(1000),
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        );
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
        // Days since start > lookback_days (365) -> No seed weight
        let pivot = rt.block_on(engine.compute_pivot(&[], &historical_trades, None, 366));

        // VWAP: (100*10 + 110*10) / 20 = 2100 / 20 = 105
        assert_eq!(pivot, Decimal::from(105));
    }

    #[test]
    fn test_compute_pivot_seeded() {
        // Seed price 100, nominal volume 10 per day.
        // lookback_days = 10.
        // days_since_start = 5.
        // Remaining seed days = 5. Seed volume = 5 * 10 = 50.
        let engine = PivotEngine::new(
            Decimal::from(100),
            10,
            0,
            Decimal::from(10),
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        );

        let historical_trades = vec![Trade {
            id: "1".to_string(),
            timestamp: 1000,
            price: Decimal::from(120),
            volume: Decimal::from(50),
            side: OrderSide::Buy,
            wallet: "w1".to_string(),
        }];

        let rt = tokio::runtime::Runtime::new().unwrap();
        let pivot = rt.block_on(engine.compute_pivot(&[], &historical_trades, None, 5));

        // Historical: 120 * 50 = 6000 value, volume 50
        // Seed: 100 * 50 = 5000 value, volume 50
        // Total: 11000 / 100 = 110
        assert_eq!(pivot, Decimal::from(110));
    }

    #[test]
    fn test_compute_pivot_empty_data() {
        let seed = Decimal::from(1000);
        let engine = PivotEngine::new(
            seed,
            365,
            0,
            Decimal::from(1000),
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        );
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pivot = rt.block_on(engine.compute_pivot(&[], &[], None, 0));
        assert_eq!(pivot, seed);
    }

    #[test]
    fn test_compute_pivot_zero_volume_fallback() {
        let engine = PivotEngine::new(
            Decimal::ZERO,
            365,
            0,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        );
        let market_update = MarketUpdate {
            timestamp: 1000,
            price: Decimal::from(150),
            volume_24h: Decimal::ZERO,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        // Both historical and seed volume are zero
        let pivot = rt.block_on(engine.compute_pivot(&[], &[], Some(&market_update), 366));

        // Should fallback to market update price (150)
        assert_eq!(pivot, Decimal::from(150));
    }

    #[test]
    fn test_compute_pivot_with_costs_and_fees() {
        let engine = PivotEngine::new(
            Decimal::ZERO,
            365,
            0,
            Decimal::from(1000),
            Decimal::from(1),    // 1 SOL market rent
            Decimal::from(0.5),  // 0.5 SOL account rent
            Decimal::from(0.25), // 0.25 SOL tip
            Decimal::from(25),   // 0.25% fee
        );
        let historical_trades = vec![Trade {
            id: "1".to_string(),
            timestamp: 1000,
            price: Decimal::from(100),
            volume: Decimal::from(10),
            side: OrderSide::Buy,
            wallet: "w1".to_string(),
        }];

        let rt = tokio::runtime::Runtime::new().unwrap();
        let pivot = rt.block_on(engine.compute_pivot(&[], &historical_trades, None, 366));

        // Base: 100 * 10 = 1000, volume 10
        // Cost: 1.75 SOL * 100 = 175
        // Fee: 10 * 0.0025 = 0.025
        // Pivot: (1000 + 175) / (10 - 0.025) = 1175 / 9.975 = 117.79...
        let expected = Decimal::from_str_exact("117.7894736842105263157894737").unwrap();
        assert_eq!(pivot, expected);
    }
}
