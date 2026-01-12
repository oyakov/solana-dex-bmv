use crate::infra::{SolanaProvider, WalletManager};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::Decimal;
use tracing::info;

pub struct RentRecoveryService {
    solana: std::sync::Arc<dyn SolanaProvider>,
    wallet_manager: std::sync::Arc<WalletManager>,
    _settings: BotSettings,
}

impl RentRecoveryService {
    pub fn new(
        solana: std::sync::Arc<dyn SolanaProvider>,
        wallet_manager: std::sync::Arc<WalletManager>,
        settings: BotSettings,
    ) -> Self {
        Self {
            solana,
            wallet_manager,
            _settings: settings,
        }
    }

    pub async fn recover_rent(&self) -> Result<()> {
        info!("Rent Recovery Service: scanning for closed accounts");

        let wallets = self.wallet_manager.get_all_wallets();
        let total_reclaimed = Decimal::ZERO;

        for _wallet in wallets {
            // TODO: Implementation for Phase 2:
            // 1. Scan for closed order accounts (OpenBook)
            // 2. Execute CloseAccount instruction via Jito
            // 3. Track reclaimed SOL
        }

        if !total_reclaimed.is_zero() {
            info!(total = %total_reclaimed, "Rent Recovery: SOL reclaimed from closed accounts");
        }

        Ok(())
    }
}
