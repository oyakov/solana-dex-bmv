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
