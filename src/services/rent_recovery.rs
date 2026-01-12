use crate::infra::{SolanaProvider, WalletManager};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
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

        let market_id = &self._settings.openbook_market_id;
        let wallets = self.wallet_manager.get_all_wallets();
        let mut total_reclaimed = Decimal::ZERO;

        for wallet in wallets {
            // Find OpenOrders account for this market
            if let Ok(Some(oo_pubkey)) = self
                .solana
                .find_open_orders(market_id, &wallet.pubkey())
                .await
            {
                // In a production bot, we would check if it has 0 orders.
                // For Phase 2, we implement the ability to close it.
                // To avoid breaking the bot's current grid, we might only close it
                // if it's explicitly marked as "to be closed" or if we are rotating wallets.

                // For now, let's just log and provide the implementation.
                info!(wallet = %wallet.pubkey(), %oo_pubkey, "Found OpenOrders account. (Closing placeholder for Phase 2)");

                // Example of actual closure:
                // let sig = self.solana.close_open_orders_account(&wallet, &oo_pubkey).await?;
                // info!(%sig, "Closed OpenOrders account and reclaimed rent");
                total_reclaimed += Decimal::new(23, 3); // ~0.023 SOL
            }
        }

        if !total_reclaimed.is_zero() {
            info!(total = %total_reclaimed, "Rent Recovery: SOL available for reclamation from detected accounts");
        }

        Ok(())
    }
}
