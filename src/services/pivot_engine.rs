use crate::domain::{AssetPosition, MarketUpdate, Trade};
use rust_decimal::prelude::*;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug)]
pub struct PivotEngine {
    pub seed_price: Decimal,
    pub lookback_minutes: u32,
    pub lookback_days: u32,
    pub nominal_daily_volume: Decimal,
    pub market_id_rent_sol: Decimal,
    pub account_rent_sol: Decimal,
    pub jito_tip_sol: Decimal,
    pub fee_bps: Decimal,

    // In-memory trade cache for responsiveness
    trade_cache: RwLock<VecDeque<Trade>>,
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
            trade_cache: RwLock::new(VecDeque::with_capacity(1000)),
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
        elapsed_seconds: i64,
    ) -> Decimal {
        let mut total_value = Decimal::ZERO;
        let mut total_volume = Decimal::ZERO;

        // 1. Calculate from historical trades (from DB)
        for trade in historical_trades {
            total_value += trade.price * trade.volume;
            total_volume += trade.volume;
        }

        // 2. Include in-memory cached trades (for real-time responsiveness)
        let cached = self.cached_trades().await;
        for trade in cached {
            // Avoid double counting if trade is already in historical_trades
            if !historical_trades.iter().any(|t| t.id == trade.id) {
                total_value += trade.price * trade.volume;
                total_volume += trade.volume;
            }
        }

        // 3. Include current market data (last 24h) if available
        if let Some(update) = market_update {
            total_value += update.price * update.volume_24h;
            total_volume += update.volume_24h;
        }

        // 4. Add Seeded Pivot weight
        let lookback_days = if self.lookback_minutes > 0 {
            Decimal::from(self.lookback_window_seconds() as u64) / Decimal::from(86_400)
        } else {
            Decimal::from(self.lookback_days)
        };
        let elapsed_days = Decimal::from(elapsed_seconds.max(0) as u64) / Decimal::from(86_400);
        let remaining_days = if elapsed_days < lookback_days {
            lookback_days - elapsed_days
        } else {
            Decimal::ZERO
        };

        if remaining_days > Decimal::ZERO {
            let seed_price = if self.seed_price.is_zero() {
                market_update.map(|m| m.price).unwrap_or(Decimal::from(100))
            } else {
                self.seed_price
            };
            let seed_volume = remaining_days * self.nominal_daily_volume;

            total_value += seed_price * seed_volume;
            total_volume += seed_volume;

            info!(
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
        let pivot = rt.block_on(engine.compute_pivot(&[], &historical_trades, None, 366 * 86_400));
        assert_eq!(pivot, Decimal::from(105));
    }
}
