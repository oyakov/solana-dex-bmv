use crate::infra::{SolanaProvider, WalletManager};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use tracing::info;

pub struct RebalanceService {
    solana: std::sync::Arc<dyn crate::infra::SolanaProvider>,
    wallet_manager: std::sync::Arc<WalletManager>,
    settings: std::sync::Arc<tokio::sync::RwLock<BotSettings>>,
    last_pivot: std::sync::Mutex<Option<Decimal>>,
    last_rebuild: std::sync::Mutex<Option<std::time::Instant>>,
    last_grid: std::sync::Mutex<Vec<crate::domain::GridLevel>>,
}

impl RebalanceService {
    pub fn new(
        solana: std::sync::Arc<dyn SolanaProvider>,
        wallet_manager: std::sync::Arc<WalletManager>,
        settings: std::sync::Arc<tokio::sync::RwLock<BotSettings>>,
    ) -> Self {
        Self {
            solana,
            wallet_manager,
            settings,
            last_pivot: std::sync::Mutex::new(None),
            last_rebuild: std::sync::Mutex::new(None),
            last_grid: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn update_last_grid(&self, grid: Vec<crate::domain::GridLevel>) {
        let mut last_grid_lock = self.last_grid.lock().unwrap();
        *last_grid_lock = grid;
    }

    pub async fn should_rebuild(&self, current_pivot: Decimal, current_price: Decimal) -> bool {
        let now = std::time::Instant::now();

        // 1. Mandatory hourly sync (3600 seconds) - use a temporary block to drop lock
        {
            let mut last_rebuild_lock = self.last_rebuild.lock().unwrap();
            if let Some(last_time) = *last_rebuild_lock {
                if now.duration_since(last_time).as_secs() >= 3600 {
                    info!("Mandatory hourly sync triggered");
                    let mut last_pivot_lock = self.last_pivot.lock().unwrap();
                    *last_pivot_lock = Some(current_pivot);
                    *last_rebuild_lock = Some(now);
                    return true;
                }
            } else {
                // First time initialization
                info!("First grid initialization");
                let mut last_pivot_lock = self.last_pivot.lock().unwrap();
                *last_pivot_lock = Some(current_pivot);
                *last_rebuild_lock = Some(now);
                return true;
            }
        }

        // 2. Threshold-based rebalance
        let threshold = {
            let s = self.settings.read().await;
            s.order_grid.rebalance_threshold_percent / Decimal::from(100)
        };

        let mut last_pivot_lock = self.last_pivot.lock().unwrap();
        if let Some(last_p) = *last_pivot_lock {
            if last_p.is_zero() {
                *last_pivot_lock = Some(current_pivot);
                return true;
            }
            let diff = (current_pivot - last_p).abs() / last_p;
            if diff > threshold {
                info!(
                    ?diff,
                    ?threshold,
                    ?last_p,
                    ?current_pivot,
                    "Rebalance threshold triggered"
                );
                *last_pivot_lock = Some(current_pivot);
                if let Ok(mut last_rebuild_lock) = self.last_rebuild.lock() {
                    *last_rebuild_lock = Some(now);
                }
                return true;
            }
        }

        // 3. Proximity-based rebalance (v2.7 Requirement)
        // If orders are too close to market price ( < 3%)
        let last_grid_lock = self.last_grid.lock().unwrap();
        if !last_grid_lock.is_empty() {
            let proximity_threshold = Decimal::new(3, 2); // 3%

            for level in last_grid_lock.iter() {
                if level.price.is_zero() {
                    continue;
                }
                let diff = (current_price - level.price).abs() / level.price;
                if diff < proximity_threshold {
                    info!(
                        ?diff,
                        ?proximity_threshold,
                        ?current_price,
                        order_price = ?level.price,
                        side = ?level.side,
                        "Proximity rebalance triggered (price too close to order)"
                    );
                    *last_pivot_lock = Some(current_pivot);
                    if let Ok(mut lock) = self.last_rebuild.lock() {
                        *lock = Some(now);
                    }
                    return true;
                }
            }
        }

        false
    }

    pub async fn rebalance(&self) -> Result<()> {
        // This method can still be used for periodic wallet balance checks or other housekeeping
        let wallets = self.wallet_manager.get_all_wallets().await;
        for wallet in wallets {
            let balance_lamports = self
                .solana
                .get_balance(&wallet.pubkey().to_string())
                .await?;
            let balance_sol = Decimal::from(balance_lamports) / Decimal::from(1_000_000_000u64);
            info!(wallet = %(*wallet).pubkey(), balance = %balance_sol, "Wallet health check");
        }
        Ok(())
    }
}
