use crate::domain::{GridLevel, OrderSide};
use crate::services::GridBuilder;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioType {
    UpwardSaw,
    DownwardSaw,
    FlashCrash,
    Pump,
    Flat,
    UpDownHills,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationPriceTick {
    pub timestamp: i64,
    pub price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub price_history: Vec<SimulationPriceTick>,
    pub projected_grids: Vec<Vec<GridLevel>>,
    pub density_distribution: Vec<DensityLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DensityLevel {
    pub price: Decimal,
    pub volume: Decimal,
    pub side: OrderSide,
}

pub struct SimulationEngine {
    grid_builder: GridBuilder,
}

impl SimulationEngine {
    pub fn new(grid_builder: GridBuilder) -> Self {
        Self { grid_builder }
    }

    pub async fn run_simulation(
        &self,
        scenario: ScenarioType,
        base_price: Decimal,
        steps: usize,
        volatility: Decimal,
    ) -> SimulationResult {
        let price_history = self.generate_prices(scenario, base_price, steps, volatility);
        let mut projected_grids = Vec::with_capacity(steps);

        // In a real MM bot, the grid is redeployed when price moves outside bounds.
        // For simulation, we can simulate grid updates at each step or when thresholds are met.
        // Let's build a grid at each step for visualization.
        for tick in &price_history {
            let grid = self
                .grid_builder
                .build(tick.price, Decimal::from(100))
                .await; // Using fixed total size for sim
            projected_grids.push(grid);
        }

        let density_distribution = self.calculate_density(&projected_grids);

        SimulationResult {
            price_history,
            projected_grids,
            density_distribution,
        }
    }

    fn generate_prices(
        &self,
        scenario: ScenarioType,
        base_price: Decimal,
        steps: usize,
        volatility: Decimal,
    ) -> Vec<SimulationPriceTick> {
        let mut prices = Vec::with_capacity(steps);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        for i in 0..steps {
            let t = i as f64 / steps as f64;
            let noise = self.dummy_noise(volatility);

            let trend = match scenario {
                ScenarioType::UpwardSaw => {
                    let upward = Decimal::from_f64(t * 0.2).unwrap(); // +20% trend
                    let saw = Decimal::from_f64((i % 10) as f64 * 0.005).unwrap();
                    upward + saw
                }
                ScenarioType::DownwardSaw => {
                    let downward = Decimal::from_f64(t * -0.2).unwrap(); // -20% trend
                    let saw = Decimal::from_f64((i % 10) as f64 * -0.005).unwrap();
                    downward + saw
                }
                ScenarioType::FlashCrash => {
                    if i > steps / 3 && i < steps / 2 {
                        Decimal::from_f64(-0.3).unwrap() // -30% drop
                    } else if i >= steps / 2 {
                        Decimal::from_f64(-0.1).unwrap() // Partial recovery
                    } else {
                        Decimal::ZERO
                    }
                }
                ScenarioType::Pump => {
                    Decimal::from_f64(t * t * 0.5).unwrap() // Exponential pump
                }
                ScenarioType::Flat => Decimal::ZERO,
                ScenarioType::UpDownHills => {
                    let sin = (t * std::f64::consts::PI * 4.0).sin();
                    Decimal::from_f64(sin * 0.1).unwrap()
                }
            };

            let current_price = base_price * (Decimal::ONE + trend + noise);
            prices.push(SimulationPriceTick {
                timestamp: now + (i as i64 * 60),
                price: current_price,
            });
        }

        prices
    }

    fn calculate_density(&self, grids: &[Vec<GridLevel>]) -> Vec<DensityLevel> {
        let mut density_map: std::collections::BTreeMap<(i64, OrderSide), Decimal> =
            std::collections::BTreeMap::new();

        for grid in grids {
            for level in grid {
                // Bin by price (e.g., 1% bins or fixed tick)
                // For simplicity, let's round to 2 significant digits or similar
                let binned_price = level.price.round_dp(4);
                let entry = density_map
                    .entry((binned_price.to_i64().unwrap_or(0), level.side))
                    .or_insert(Decimal::ZERO);
                *entry += level.size;
            }
        }

        density_map
            .into_iter()
            .map(|((p, s), v)| DensityLevel {
                price: Decimal::from(p),
                volume: v,
                side: s,
            })
            .collect()
    }

    fn dummy_noise(&self, volatility: Decimal) -> Decimal {
        // Simple deterministic "noise" for simulation consistency
        let n = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % 100) as f64;
        Decimal::from_f64((n - 50.0) / 500.0).unwrap() * volatility
    }
}
