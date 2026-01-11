use crate::utils::RiskLimitsSettings;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct RiskSnapshot {
    pub daily_loss_usd: Decimal,
    pub open_orders: u32,
}

#[derive(Debug, Clone)]
pub enum CircuitBreakerReason {
    MaxDailyLoss { limit: Decimal, value: Decimal },
    MaxOpenOrders { limit: u32, value: u32 },
}

impl std::fmt::Display for CircuitBreakerReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerReason::MaxDailyLoss { limit, value } => write!(
                f,
                "max daily loss exceeded: limit={} value={}",
                limit, value
            ),
            CircuitBreakerReason::MaxOpenOrders { limit, value } => write!(
                f,
                "max open orders exceeded: limit={} value={}",
                limit, value
            ),
        }
    }
}

pub struct RiskManager {
    limits: RiskLimitsSettings,
}

impl RiskManager {
    pub fn new(limits: RiskLimitsSettings) -> Self {
        Self { limits }
    }

    pub fn evaluate(&self, snapshot: &RiskSnapshot) -> Option<CircuitBreakerReason> {
        if self.limits.max_daily_loss_usd > Decimal::ZERO
            && snapshot.daily_loss_usd >= self.limits.max_daily_loss_usd
        {
            return Some(CircuitBreakerReason::MaxDailyLoss {
                limit: self.limits.max_daily_loss_usd,
                value: snapshot.daily_loss_usd,
            });
        }

        if self.limits.max_open_orders > 0 && snapshot.open_orders >= self.limits.max_open_orders {
            return Some(CircuitBreakerReason::MaxOpenOrders {
                limit: self.limits.max_open_orders,
                value: snapshot.open_orders,
            });
        }

        None
    }
}
