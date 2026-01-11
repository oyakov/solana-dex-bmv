use crate::domain::{AssetPosition, MarketUpdate, Trade};
use rust_decimal::prelude::*;
use std::collections::VecDeque;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug)]
pub struct PivotEngine {
    pub seed_price: Decimal,
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
        nominal_daily_volume: Decimal,
        market_id_rent_sol: Decimal,
        account_rent_sol: Decimal,
        jito_tip_sol: Decimal,
        fee_bps: Decimal,
    ) -> Self {
        Self {
            seed_price,
            lookback_days,
            nominal_daily_volume,
            market_id_rent_sol,
            account_rent_sol,
            jito_tip_sol,
            fee_bps,
            trade_cache: RwLock::new(VecDeque::with_capacity(1000)),
        }
    }

    pub async fn record_trade(&self, trade: Trade) {
        let mut cache = self.trade_cache.write().await;
        cache.push_back(trade);

        // Keep cache size reasonable, e.g., last 1000 trades or based on lookback
        if cache.len() > 1000 {
            cache.pop_front();
        }
    }

    pub async fn cached_trades(&self) -> Vec<Trade> {
        self.trade_cache.read().await.iter().cloned().collect()
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
        if days_since_start < self.lookback_days {
            let seed_price = if self.seed_price.is_zero() {
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
