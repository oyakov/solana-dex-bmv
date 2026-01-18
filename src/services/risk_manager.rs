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
        self.evaluate_with_limits(snapshot, &self.limits)
    }

    pub fn evaluate_with_limits(
        &self,
        snapshot: &RiskSnapshot,
        limits: &RiskLimitsSettings,
    ) -> Option<CircuitBreakerReason> {
        if limits.max_daily_loss_usd > Decimal::ZERO
            && snapshot.daily_loss_usd >= limits.max_daily_loss_usd
        {
            return Some(CircuitBreakerReason::MaxDailyLoss {
                limit: limits.max_daily_loss_usd,
                value: snapshot.daily_loss_usd,
            });
        }

        if limits.max_open_orders > 0 && snapshot.open_orders >= limits.max_open_orders {
            return Some(CircuitBreakerReason::MaxOpenOrders {
                limit: limits.max_open_orders,
                value: snapshot.open_orders,
            });
        }

        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::RiskLimitsSettings;
    use rust_decimal::Decimal;

    #[test]
    fn test_risk_manager_max_daily_loss() {
        let limits = RiskLimitsSettings {
            max_daily_loss_usd: Decimal::from(100),
            max_open_orders: 10,
            max_position_usd: Decimal::ZERO,
            max_order_usd: Decimal::ZERO,
        };
        let manager = RiskManager::new(limits);

        // Under limit
        let snapshot = RiskSnapshot {
            daily_loss_usd: Decimal::from(50),
            open_orders: 5,
        };
        assert!(manager.evaluate(&snapshot).is_none());

        // At limit
        let snapshot = RiskSnapshot {
            daily_loss_usd: Decimal::from(100),
            open_orders: 5,
        };
        let reason = manager.evaluate(&snapshot).unwrap();
        match reason {
            CircuitBreakerReason::MaxDailyLoss { limit, value } => {
                assert_eq!(limit, Decimal::from(100));
                assert_eq!(value, Decimal::from(100));
            }
            _ => panic!("Expected MaxDailyLoss"),
        }

        // Over limit
        let snapshot = RiskSnapshot {
            daily_loss_usd: Decimal::from(150),
            open_orders: 5,
        };
        assert!(manager.evaluate(&snapshot).is_some());
    }

    #[test]
    fn test_risk_manager_max_open_orders() {
        let limits = RiskLimitsSettings {
            max_daily_loss_usd: Decimal::from(1000),
            max_open_orders: 2,
            max_position_usd: Decimal::ZERO,
            max_order_usd: Decimal::ZERO,
        };
        let manager = RiskManager::new(limits);

        // Under limit
        let snapshot = RiskSnapshot {
            daily_loss_usd: Decimal::from(100),
            open_orders: 1,
        };
        assert!(manager.evaluate(&snapshot).is_none());

        // At limit
        let snapshot = RiskSnapshot {
            daily_loss_usd: Decimal::from(100),
            open_orders: 2,
        };
        let reason = manager.evaluate(&snapshot).unwrap();
        match reason {
            CircuitBreakerReason::MaxOpenOrders { limit, value } => {
                assert_eq!(limit, 2);
                assert_eq!(value, 2);
            }
            _ => panic!("Expected MaxOpenOrders"),
        }
    }

    #[test]
    fn test_risk_manager_disabled_limits() {
        let limits = RiskLimitsSettings {
            max_daily_loss_usd: Decimal::ZERO,
            max_open_orders: 0,
            max_position_usd: Decimal::ZERO,
            max_order_usd: Decimal::ZERO,
        };
        let manager = RiskManager::new(limits);

        let snapshot = RiskSnapshot {
            daily_loss_usd: Decimal::from(1000000),
            open_orders: 1000,
        };
        assert!(manager.evaluate(&snapshot).is_none());
    }
}
