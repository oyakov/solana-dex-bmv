use crate::infra::{SolanaProvider, WalletManager};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use tracing::info;

pub struct FinancialManager {
    solana: std::sync::Arc<dyn SolanaProvider>,
    wallet_manager: std::sync::Arc<WalletManager>,
    settings: BotSettings,
}

impl FinancialManager {
    pub fn new(
        solana: std::sync::Arc<dyn SolanaProvider>,
        wallet_manager: std::sync::Arc<WalletManager>,
        settings: BotSettings,
    ) -> Self {
        Self {
            solana,
            wallet_manager,
            settings,
        }
    }

    pub async fn check_balances(&self) -> Result<()> {
        info!("Financial Manager: checking SOL/USDC balances");

        // 1. Calculate aggregated balance across swarm
        let mut total_sol = Decimal::ZERO;
        let wallets = self.wallet_manager.get_all_wallets();

        for wallet in &wallets {
            let lamports = self
                .solana
                .get_balance(&wallet.pubkey().to_string())
                .await?;
            total_sol += Decimal::from(lamports) / Decimal::from(1_000_000_000u64);
        }

        info!(total_sol = %total_sol, "Aggregated swarm SOL balance");

        // 2. Check against MIN_SOL_RESERVE_% (placeholder logic)
        // In a real implementation, we would also fetch USDC balance and
        // perform conversions if needed using Jupiter DEX.

        Ok(())
    }

    pub async fn rebalance_fiat(
        &self,
        current_price: Decimal,
        grid_boundary: Decimal,
    ) -> Result<()> {
        // Logic for SOL <-> USDC conversion based on position in channel
        // e.g. Sell some SOL for USDC at upper boundary
        info!(
            %current_price,
            %grid_boundary,
            "Financial Manager: evaluating fiat/sol rebalance"
        );

        Ok(())
    }
}
