use crate::domain::{GridLevel, OrderSide};
use rust_decimal::prelude::*;
use tracing::info;

#[derive(Debug, Clone)]
pub struct GridBuilder {
    pub orders_per_side: u32,
    pub buy_channel_width: Decimal,
    pub sell_channel_width: Decimal,
}

impl Default for GridBuilder {
    fn default() -> Self {
        Self {
            orders_per_side: 16,
            buy_channel_width: Decimal::from_str_radix("0.15", 10).unwrap(),
            sell_channel_width: Decimal::from_str_radix("0.30", 10).unwrap(),
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

        // BUY orders (Support)
        let buy_step = (mid_price * self.buy_channel_width) / orders_per_side_dec;
        let size_per_order = total_size / (orders_per_side_dec * Decimal::from(2));

        for i in 1..=self.orders_per_side {
            let price = mid_price - (buy_step * Decimal::from(i));
            grid.push(GridLevel {
                price,
                size: size_per_order,
                side: OrderSide::Buy,
            });
        }

        // SELL orders (Growth)
        let sell_step = (mid_price * self.sell_channel_width) / orders_per_side_dec;
        for i in 1..=self.orders_per_side {
            let price = mid_price + (sell_step * Decimal::from(i));
            grid.push(GridLevel {
                price,
                size: size_per_order,
                side: OrderSide::Sell,
            });
        }

        info!(
            ?mid_price,
            buy_levels = self.orders_per_side,
            sell_levels = self.orders_per_side,
            buy_width = ?self.buy_channel_width,
            sell_width = ?self.sell_channel_width,
            "grid_built"
        );

        grid
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
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let grid = rt.block_on(builder.build(Decimal::from(100), Decimal::from(10)));

        // Buy price: 100 - (100 * 0.10 / 1) * 1 = 90
        // Sell price: 100 + (100 * 0.20 / 1) * 1 = 120
        assert!(grid
            .iter()
            .any(|l| l.price == Decimal::from(90) && matches!(l.side, OrderSide::Buy)));
        assert!(grid
            .iter()
            .any(|l| l.price == Decimal::from(120) && matches!(l.side, OrderSide::Sell)));
    }
}
