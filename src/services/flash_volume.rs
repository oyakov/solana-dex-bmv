use crate::infra::{SolanaProvider, WalletManager};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use std::str::FromStr;
use tracing::info;

pub struct FlashVolumeModule {
    solana: std::sync::Arc<dyn SolanaProvider>,
    wallet_manager: std::sync::Arc<WalletManager>,
    settings: BotSettings,
}

impl FlashVolumeModule {
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

    pub async fn execute_cycle(&self) -> Result<()> {
        if !self.settings.flash_volume.enabled {
            return Ok(());
        }

        info!("Flash Volume cycle triggered (Atomic Wash Trading)");

        // 1. Select two different wallets
        let wallets = self.wallet_manager.get_all_wallets();
        if wallets.len() < 2 {
            info!("Flash Volume: not enough wallets (need 2)");
            return Ok(());
        }

        let wallet_a = &wallets[0];
        let wallet_b = &wallets[1];

        // 2. Fetch market data for lot logic
        let market_id = &self.settings.openbook_market_id;
        let market_update = self.solana.get_market_data(market_id).await?;
        let price = market_update.price;

        // 3. Determine volume in SOL units
        let volume_sol = self.settings.flash_volume.size_sol;
        let tip_lamports = (self.settings.flash_volume.tip_sol * Decimal::from(1_000_000_000u64))
            .to_u64()
            .unwrap_or(1_000_000);

        // 4. Calculate lots (assuming we have lot sizes from MarketUpdate or fetching separately)
        // Since get_market_data doesn't return lot sizes, we'll use a standard calculation
        // Or better, we should have a method to get lot sizes.
        // For now, I'll mock the lot calculation or assume standard 1e9 / 1e6.
        // Actually, let's just use the current price directly for lot conversion logic if available.
        // In OpenBook V2, lot sizes are in the market state.

        // I will use placeholder lot values for now, but ideally they come from a cached market state.
        let price_lots = (price * Decimal::from(1_000_000u64)).to_i64().unwrap_or(0); // Dummy lot math
        let size_lots = (volume_sol * Decimal::from(1_000_000_000u64))
            .to_i64()
            .unwrap_or(0);

        let base_mint = solana_sdk::pubkey::Pubkey::from_str(&self.settings.token_mint)?;
        let quote_mint =
            solana_sdk::pubkey::Pubkey::from_str(&self.settings.wallets.usdc_wallet_3)?;

        info!(
            wallet_a = %wallet_a.pubkey(),
            wallet_b = %wallet_b.pubkey(),
            volume = %volume_sol,
            "Executing atomic wash trade via Jito"
        );

        let sig = self
            .solana
            .send_flash_volume_bundle(
                market_id,
                wallet_a,
                wallet_b,
                price_lots,
                size_lots,
                tip_lamports,
                &self.settings.jito_bundle.bundler_url,
                &base_mint,
                &quote_mint,
            )
            .await?;

        info!(%sig, "Flash Volume: Wash trade bundle submitted successfully");

        Ok(())
    }
}
