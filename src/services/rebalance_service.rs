use crate::infra::{SolanaClient, WalletManager};
use crate::utils::BotSettings;
use anyhow::Result;
use tracing::info;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;

pub struct RebalanceService {
    solana: std::sync::Arc<SolanaClient>,
    wallet_manager: std::sync::Arc<WalletManager>,
    _settings: BotSettings,
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
            _settings: settings,
        }
    }

    pub async fn rebalance(&self) -> Result<()> {
        info!("Starting rebalancing check...");
        
        let wallets = self.wallet_manager.get_all_wallets();
        for wallet in wallets {
            let balance_lamports = self.solana.get_balance(&wallet.pubkey().to_string()).await?;
            let balance_sol = Decimal::from(balance_lamports) / Decimal::from(1_000_000_000u64);
            
            info!(wallet = %wallet.pubkey(), balance = %balance_sol, "Wallet status");
        }
        
        Ok(())
    }
}
