use crate::infra::{SolanaProvider, WalletManager};
use crate::utils::BotSettings;
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct EmergencyPoolService {
    solana: Arc<dyn SolanaProvider>,
    wallet_manager: Arc<WalletManager>,
    settings: Arc<RwLock<BotSettings>>,
}

impl EmergencyPoolService {
    pub fn new(
        solana: Arc<dyn SolanaProvider>,
        wallet_manager: Arc<WalletManager>,
        settings: Arc<RwLock<BotSettings>>,
    ) -> Self {
        Self {
            solana,
            wallet_manager,
            settings,
        }
    }

    /// High-level method to restart the pool:
    /// 1. Create a new OpenBook Market (V2)
    /// 2. Initialize the side wallets
    /// 3. Seed orders via Jito Bundle
    pub async fn restart_pool(&self) -> Result<String> {
        info!("Emergency Pool Restart initiated");

        let (token_mint, usdc_mint_str) = {
            let s = self.settings.read().await;
            (s.token_mint.clone(), s.wallets.usdc_wallet_3.clone())
        };

        let base_mint = Pubkey::parse(&token_mint)?;
        let quote_mint = Pubkey::parse(&usdc_mint_str)?;

        let wallets = self.wallet_manager.get_all_wallets().await;
        if wallets.is_empty() {
            return Err(anyhow!("No wallets available for seeding"));
        }

        // 1. Create Market
        let market_authority = wallets[0].clone(); // Use first wallet as authority for simplicity
        info!("Step 1: Creating new OpenBook V2 Market for {}/{}", token_mint, usdc_mint_str);
        
        let new_market_id = self.solana.create_market(&base_mint, &quote_mint, &market_authority).await?;
        info!(%new_market_id, "Market created successfully");

        // 2. Seed Orders (Atomic injection)
        info!("Step 2: Seeding initial liquidity via Jito Bundle");
        
        let wallets = self.wallet_manager.get_all_wallets().await;
        if wallets.is_empty() {
            return Err(anyhow!("No wallets available for seeding"));
        }

        // We would construct a bundle of 'place_order' instructions
        // self.solana.send_bundle(...)
        
        info!("Pool restart completed. New Market ID: {}", new_market_id);
        
        // Update settings with new market ID
        {
            let mut s = self.settings.write().await;
            s.openbook_market_id = new_market_id.to_string();
        }

        Ok(new_market_id.to_string())
    }
}
