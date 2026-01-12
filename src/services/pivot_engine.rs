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
    last_pivot: RwLock<Decimal>,
}

impl PivotEngine {
    #[allow(clippy::too_many_arguments)]
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
            last_pivot: RwLock::new(seed_price),
        }
    }

    pub async fn get_last_pivot(&self) -> Decimal {
        *self.last_pivot.read().await
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

        let pivot = if total_volume.is_zero() {
            warn!("no_volume_detected_falling_back_to_seed_or_market");
            if !self.seed_price.is_zero() {
                self.seed_price
            } else {
                market_update.map(|m| m.price).unwrap_or(Decimal::ZERO)
            }
        } else {
            total_value / total_volume
        };

        let current_price = market_update.map(|m| m.price).unwrap_or(Decimal::ZERO);
        info!(
            ?total_volume,
            ?total_value,
            ?pivot,
            ?current_price,
            "Pivot computed (Pure VWAP)"
        );

        *self.last_pivot.write().await = pivot;
        pivot
    }

    pub async fn set_last_price(&self, price: Decimal) {
        *self.last_pivot.write().await = price;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::OrderSide;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_trade_cache_pruning() {
        let engine = PivotEngine::new(
            Decimal::ZERO,
            0,
            1, // 1 minute lookback
            Decimal::from(1000),
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Trade outside lookback (push first)
            engine
                .record_trade(Trade {
                    id: "old".to_string(),
                    timestamp: now - 120, // 2m ago
                    price: Decimal::from(50),
                    volume: Decimal::from(1),
                    side: OrderSide::Buy,
                    wallet: "w1".to_string(),
                })
                .await;

            // Trade within lookback
            engine
                .record_trade(Trade {
                    id: "recent".to_string(),
                    timestamp: now - 30, // 30s ago
                    price: Decimal::from(100),
                    volume: Decimal::from(1),
                    side: OrderSide::Buy,
                    wallet: "w1".to_string(),
                })
                .await;

            let cached = engine.cached_trades().await;
            assert_eq!(cached.len(), 1);
            assert_eq!(cached[0].id, "recent");
        });
    }

    #[test]
    fn test_compute_pivot_with_cache_no_double_count() {
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

        let trade = Trade {
            id: "1".to_string(),
            timestamp: 1000,
            price: Decimal::from(100),
            volume: Decimal::from(10),
            side: OrderSide::Buy,
            wallet: "w1".to_string(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Seed the same trade in cache
            engine.seed_trades(vec![trade.clone()]).await;

            let historical_trades = vec![trade];

            // Pivot should be 100, not skewed if double counting happened
            // (100 * 10) / 10 = 100
            let pivot = engine
                .compute_pivot(&[], &historical_trades, None, 366 * 86_400)
                .await;
            assert_eq!(pivot, Decimal::from(100));
        });
    }
}
