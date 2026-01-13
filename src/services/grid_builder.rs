use crate::domain::{GridLevel, OrderSide};
use rust_decimal::prelude::*;
use tracing::info;

#[derive(Debug, Clone)]
pub struct GridBuilder {
    pub orders_per_side: u32,
    pub buy_channel_width: Decimal,
    pub sell_channel_width: Decimal,
    pub buy_volume_multiplier: Decimal,
    pub sell_volume_multiplier: Decimal,
}

impl Default for GridBuilder {
    fn default() -> Self {
        Self {
            orders_per_side: 16,
            buy_channel_width: Decimal::from_str_radix("0.15", 10).unwrap(),
            sell_channel_width: Decimal::from_str_radix("0.30", 10).unwrap(),
            buy_volume_multiplier: Decimal::new(12, 1), // 1.2
            sell_volume_multiplier: Decimal::ONE,       // 1.0
        }
    }
}

impl GridBuilder {
    pub async fn build(&self, mid_price: Decimal, total_size: Decimal) -> Vec<GridLevel> {
        if self.orders_per_side == 0 {
            return Vec::new();
        }

        let mut grid = Vec::with_capacity((self.orders_per_side * 2) as usize);
        let orders_per_side_dec = Decimal::from(self.orders_per_side);

        // 1. BUY orders (Support)
        let buy_step = (mid_price * self.buy_channel_width) / orders_per_side_dec;
        let buy_side_total = total_size / Decimal::from(2);

        // Calculate weights for exponential distribution
        let mut buy_weights = Vec::with_capacity(self.orders_per_side as usize);
        let mut buy_total_weight = Decimal::ZERO;
        let mut current_buy_weight = Decimal::ONE;
        for _ in 0..self.orders_per_side {
            buy_weights.push(current_buy_weight);
            buy_total_weight += current_buy_weight;
            current_buy_weight *= self.buy_volume_multiplier;
        }

        for (idx, weight) in buy_weights.into_iter().enumerate() {
            let i = (idx + 1) as u32;
            let price = mid_price - (buy_step * Decimal::from(i));
            let size = if buy_total_weight.is_zero() {
                Decimal::ZERO
            } else {
                (buy_side_total / buy_total_weight) * weight
            };
            grid.push(GridLevel {
                price,
                size,
                side: OrderSide::Buy,
            });
        }

        // 2. SELL orders (Growth)
        let sell_step = (mid_price * self.sell_channel_width) / orders_per_side_dec;
        let sell_side_total = total_size / Decimal::from(2);

        // Calculate weights for exponential distribution
        let mut sell_weights = Vec::with_capacity(self.orders_per_side as usize);
        let mut sell_total_weight = Decimal::ZERO;
        let mut current_sell_weight = Decimal::ONE;
        for _ in 0..self.orders_per_side {
            sell_weights.push(current_sell_weight);
            sell_total_weight += current_sell_weight;
            current_sell_weight *= self.sell_volume_multiplier;
        }

        for (idx, weight) in sell_weights.into_iter().enumerate() {
            let i = (idx + 1) as u32;
            let price = mid_price + (sell_step * Decimal::from(i));
            let size = if sell_total_weight.is_zero() {
                Decimal::ZERO
            } else {
                (sell_side_total / sell_total_weight) * weight
            };
            grid.push(GridLevel {
                price,
                size,
                side: OrderSide::Sell,
            });
        }

        info!(
            ?mid_price,
            buy_levels = self.orders_per_side,
            sell_levels = self.orders_per_side,
            buy_width = ?self.buy_channel_width,
            sell_width = ?self.sell_channel_width,
            buy_mult = ?self.buy_volume_multiplier,
            sell_mult = ?self.sell_volume_multiplier,
            "grid_built"
        );

        grid
    }

    pub fn apply_front_running_protection(
        &self,
        levels: &mut [GridLevel],
        orderbook: &crate::domain::Orderbook,
        large_order_threshold: Decimal,
        tick_size: Decimal,
    ) {
        for level in levels.iter_mut() {
            match level.side {
                OrderSide::Buy => {
                    // Find the best bid that is >= large_order_threshold and slightly above or near our price
                    // We want to be 1-tick ahead of the highest large competitor that is BELOW the mid-price
                    if let Some(competitor) = orderbook.bids.iter().find(|b| {
                        b.size >= large_order_threshold
                            && b.price <= level.price * Decimal::from_str_radix("1.05", 10).unwrap()
                    }) {
                        let new_price = competitor.price + tick_size;
                        if new_price < level.price * Decimal::from_str_radix("1.1", 10).unwrap() {
                            level.price = new_price;
                        }
                    }
                }
                OrderSide::Sell => {
                    // Find the best ask that is >= large_order_threshold and slightly below or near our price
                    if let Some(competitor) = orderbook.asks.iter().find(|a| {
                        a.size >= large_order_threshold
                            && a.price >= level.price * Decimal::from_str_radix("0.95", 10).unwrap()
                    }) {
                        let new_price = competitor.price - tick_size;
                        if new_price > level.price * Decimal::from_str_radix("0.9", 10).unwrap() {
                            level.price = new_price;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_builder_buy_sell_count() {
        let builder = GridBuilder {
            orders_per_side: 5,
            buy_channel_width: Decimal::from_str_radix("0.1", 10).unwrap(),
            sell_channel_width: Decimal::from_str_radix("0.1", 10).unwrap(),
            buy_volume_multiplier: Decimal::ONE,
            sell_volume_multiplier: Decimal::ONE,
        };

        // Using tokio::test would be better if build was truly async,
        // but it doesn't await anything currently, so we can use a runtime block if needed
        // or just make it sync if it's pure logic.
        // For now, let's assume we can run it sync for testing logic.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let grid = rt.block_on(builder.build(Decimal::from(100), Decimal::from(10)));

        assert_eq!(grid.len(), 10);
        let buys = grid
            .iter()
            .filter(|l| matches!(l.side, OrderSide::Buy))
            .count();
        let sells = grid
            .iter()
            .filter(|l| matches!(l.side, OrderSide::Sell))
            .count();
        assert_eq!(buys, 5);
        assert_eq!(sells, 5);
    }

    #[test]
    fn test_grid_prices() {
        let builder = GridBuilder {
            orders_per_side: 1,
            buy_channel_width: Decimal::from_str_radix("0.10", 10).unwrap(),
            sell_channel_width: Decimal::from_str_radix("0.20", 10).unwrap(),
            buy_volume_multiplier: Decimal::ONE,
            sell_volume_multiplier: Decimal::ONE,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let grid = rt.block_on(builder.build(Decimal::from(100), Decimal::from(10)));

        // Buy price: 100 - (100 * 0.10 / 1) * 1 = 90
        // Sell price: 100 + (100 * 0.20 / 1) * 1 = 120
        assert!(grid
            .iter()
            .any(|l| l.price == Decimal::from(120) && matches!(l.side, OrderSide::Sell)));
    }

    #[test]
    fn test_grid_zero_orders() {
        let builder = GridBuilder {
            orders_per_side: 0,
            ..Default::default()
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let grid = rt.block_on(builder.build(Decimal::from(100), Decimal::from(10)));
        assert_eq!(grid.len(), 0);
    }

    #[test]
    fn test_grid_spacing() {
        let builder = GridBuilder {
            orders_per_side: 2,
            buy_channel_width: Decimal::from_str_radix("0.10", 10).unwrap(), // 10%
            sell_channel_width: Decimal::from_str_radix("0.20", 10).unwrap(), // 20%
            buy_volume_multiplier: Decimal::ONE,
            sell_volume_multiplier: Decimal::ONE,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let grid = rt.block_on(builder.build(Decimal::from(100), Decimal::from(10)));

        // Buy Step: (100 * 0.10) / 2 = 5
        // Buy Levels: 100 - 5 = 95, 100 - 10 = 90
        // Sell Step: (100 * 0.20) / 2 = 10
        // Sell Levels: 100 + 10 = 110, 100 + 20 = 120

        let mut prices: Vec<Decimal> = grid.iter().map(|l| l.price).collect();
        prices.sort();

        assert_eq!(
            prices,
            vec![
                Decimal::from(90),
                Decimal::from(95),
                Decimal::from(110),
                Decimal::from(120)
            ]
        );
    }
}
