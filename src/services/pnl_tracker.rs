use crate::domain::OrderSide;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct PnlSnapshot {
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Decimal,
    pub net_position: Decimal,
    pub average_cost: Decimal,
}

pub struct PnlTracker {
    net_position: Decimal,
    average_cost: Decimal,
    realized_pnl: Decimal,
}

impl Default for PnlTracker {
    fn default() -> Self {
        Self {
            net_position: Decimal::ZERO,
            average_cost: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
        }
    }
}

impl PnlTracker {
    pub fn record_trade(&mut self, side: OrderSide, price: Decimal, volume: Decimal) {
        if volume <= Decimal::ZERO {
            return;
        }

        match side {
            OrderSide::Buy => self.apply_buy(price, volume),
            OrderSide::Sell => self.apply_sell(price, volume),
        }
    }

    pub fn snapshot(&self, current_price: Decimal) -> PnlSnapshot {
        let unrealized_pnl = match self.net_position.to_f64() {
            Some(pos) if pos > 0.0 => (current_price - self.average_cost) * self.net_position,
            Some(pos) if pos < 0.0 => (self.average_cost - current_price) * self.net_position.abs(),
            _ => Decimal::ZERO,
        };

        PnlSnapshot {
            realized_pnl: self.realized_pnl,
            unrealized_pnl,
            net_position: self.net_position,
            average_cost: self.average_cost,
        }
    }

    fn apply_buy(&mut self, price: Decimal, volume: Decimal) {
        if self.net_position < Decimal::ZERO {
            let cover = volume.min(self.net_position.abs());
            self.realized_pnl += (self.average_cost - price) * cover;
            self.net_position += cover;

            let remaining = volume - cover;
            if remaining > Decimal::ZERO {
                let total_cost = price * remaining;
                self.average_cost = total_cost / remaining;
                self.net_position += remaining;
            } else if self.net_position == Decimal::ZERO {
                self.average_cost = Decimal::ZERO;
            }
        } else if self.net_position == Decimal::ZERO {
            self.average_cost = price;
            self.net_position = volume;
        } else {
            let total_cost = self.average_cost * self.net_position + price * volume;
            self.net_position += volume;
            self.average_cost = total_cost / self.net_position;
        }
    }

    fn apply_sell(&mut self, price: Decimal, volume: Decimal) {
        if self.net_position > Decimal::ZERO {
            let close = volume.min(self.net_position);
            self.realized_pnl += (price - self.average_cost) * close;
            self.net_position -= close;

            let remaining = volume - close;
            if remaining > Decimal::ZERO {
                let total_cost = price * remaining;
                self.average_cost = total_cost / remaining;
                self.net_position -= remaining;
            } else if self.net_position == Decimal::ZERO {
                self.average_cost = Decimal::ZERO;
            }
        } else if self.net_position == Decimal::ZERO {
            self.average_cost = price;
            self.net_position = -volume;
        } else {
            let total_cost = self.average_cost * self.net_position.abs() + price * volume;
            self.net_position -= volume;
            self.average_cost = total_cost / self.net_position.abs();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_long_realized_and_unrealized() {
        let mut tracker = PnlTracker::default();
        tracker.record_trade(OrderSide::Buy, Decimal::from(100), Decimal::from(2));
        tracker.record_trade(OrderSide::Buy, Decimal::from(110), Decimal::from(2));

        let snapshot = tracker.snapshot(Decimal::from(120));
        assert_eq!(snapshot.net_position, Decimal::from(4));
        assert_eq!(snapshot.average_cost, Decimal::from(105));
        assert_eq!(snapshot.unrealized_pnl, Decimal::from(60));

        tracker.record_trade(OrderSide::Sell, Decimal::from(130), Decimal::from(1));
        let snapshot = tracker.snapshot(Decimal::from(120));
        assert_eq!(snapshot.realized_pnl, Decimal::from(25));
        assert_eq!(snapshot.net_position, Decimal::from(3));
    }

    #[test]
    fn tracks_short_positions() {
        let mut tracker = PnlTracker::default();
        tracker.record_trade(OrderSide::Sell, Decimal::from(200), Decimal::from(1));

        let snapshot = tracker.snapshot(Decimal::from(180));
        assert_eq!(snapshot.net_position, Decimal::from(-1));
        assert_eq!(snapshot.unrealized_pnl, Decimal::from(20));

        tracker.record_trade(OrderSide::Buy, Decimal::from(190), Decimal::from(1));
        let snapshot = tracker.snapshot(Decimal::from(190));
        assert_eq!(snapshot.realized_pnl, Decimal::from(10));
        assert_eq!(snapshot.net_position, Decimal::ZERO);
    }
}
