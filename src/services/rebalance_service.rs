use crate::infra::{SolanaClient, WalletManager};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use tracing::info;

pub struct RebalanceService {
    solana: std::sync::Arc<SolanaClient>,
    wallet_manager: std::sync::Arc<WalletManager>,
    settings: BotSettings,
    last_pivot: std::sync::Mutex<Option<Decimal>>,
    last_rebuild: std::sync::Mutex<Option<std::time::Instant>>,
}

impl RebalanceService {
    pub fn new(
        solana: std::sync::Arc<SolanaClient>,
        wallet_manager: std::sync::Arc<WalletManager>,
        settings: BotSettings,
    ) -> Self {
        Self {
            solana,
            wallet_manager,
            settings,
            last_pivot: std::sync::Mutex::new(None),
            last_rebuild: std::sync::Mutex::new(None),
        }
    }

    pub fn should_rebuild(&self, current_pivot: Decimal) -> bool {
        let mut last_pivot_lock = self.last_pivot.lock().unwrap();
        let mut last_rebuild_lock = self.last_rebuild.lock().unwrap();

        let now = std::time::Instant::now();

        // 1. Mandatory hourly sync (3600 seconds)
        if let Some(last_time) = *last_rebuild_lock {
            if now.duration_since(last_time).as_secs() >= 3600 {
                info!("Mandatory hourly sync triggered");
                *last_pivot_lock = Some(current_pivot);
                *last_rebuild_lock = Some(now);
                return true;
            }
        } else {
            // First time initialization
            info!("First grid initialization");
            *last_pivot_lock = Some(current_pivot);
            *last_rebuild_lock = Some(now);
            return true;
        }

        // 2. Threshold-based rebalance
        if let Some(last_p) = *last_pivot_lock {
            let threshold =
                self.settings.order_grid.rebalance_threshold_percent / Decimal::from(100);
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
                *last_rebuild_lock = Some(now);
                return true;
            }
        }

        false
    }

    pub async fn rebalance(&self) -> Result<()> {
        // This method can still be used for periodic wallet balance checks or other housekeeping
        let wallets = self.wallet_manager.get_all_wallets();
        for wallet in wallets {
            let balance_lamports = self
                .solana
                .get_balance(&wallet.pubkey().to_string())
                .await?;
            let balance_sol = Decimal::from(balance_lamports) / Decimal::from(1_000_000_000u64);
            info!(wallet = %wallet.pubkey(), balance = %balance_sol, "Wallet health check");
        }
        Ok(())
    }
}
