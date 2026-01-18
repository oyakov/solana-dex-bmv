use crate::infra::{SolanaProvider, WalletManager};
use crate::utils::BotSettings;
use anyhow::Result;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use tracing::{info, warn};

pub struct RentRecoveryService {
    solana: std::sync::Arc<dyn SolanaProvider>,
    wallet_manager: std::sync::Arc<WalletManager>,
    _settings: std::sync::Arc<tokio::sync::RwLock<BotSettings>>,
}

impl RentRecoveryService {
    pub fn new(
        solana: std::sync::Arc<dyn SolanaProvider>,
        wallet_manager: std::sync::Arc<WalletManager>,
        _settings: std::sync::Arc<tokio::sync::RwLock<BotSettings>>,
    ) -> Self {
        Self {
            solana,
            wallet_manager,
            _settings,
        }
    }

    pub async fn recover_rent(&self) -> Result<()> {
        info!("Rent Recovery Service: scanning for closed accounts");

        let market_id = {
            let s = self._settings.read().await;
            s.openbook_market_id.clone()
        };
        let wallets = self.wallet_manager.get_all_wallets().await;
        let mut total_reclaimed = Decimal::ZERO;

        for wallet in wallets {
            let owner = wallet.pubkey();
            if let Ok(Some(oo_pubkey)) = self.solana.find_open_orders(market_id, &owner).await {
                // Safety check: Fetch account data to ensure it's empty
                match self.solana.get_open_orders_account_data(&oo_pubkey).await {
                    Ok(data) => {
                        if self.is_open_orders_empty(&data) {
                            info!(wallet = %owner, %oo_pubkey, "Closing empty OpenOrders account");
                            match self.solana.close_open_orders_account(&wallet, &oo_pubkey).await {
                                Ok(sig) => {
                                    info!(%sig, "Closed OpenOrders account and reclaimed rent");
                                    total_reclaimed += Decimal::new(23, 3); // ~0.023 SOL
                                }
                                Err(e) => warn!(error = %e, %oo_pubkey, "Failed to close OpenOrders account"),
                            }
                        } else {
                            info!(wallet = %owner, %oo_pubkey, "OpenOrders account not empty, skipping closure");
                        }
                    }
                    Err(e) => warn!(error = %e, %oo_pubkey, "Failed to fetch OpenOrders account data"),
                }
            }
        }

        if !total_reclaimed.is_zero() {
            info!(total = %total_reclaimed, "Rent Recovery: Successfully reclaimed SOL");
        }

        Ok(())
    }

    fn is_open_orders_empty(&self, data: &[u8]) -> bool {
        // OpenBook V2 OpenOrders account check
        // Rough layout: Discriminator (8) + Owner (32) + Market (32) + 
        // free_slot_bits (16) + is_free (16) + position (huge)
        
        if data.len() < 128 {
            return false;
        }

        // In OpenBook V2, if free_slot_bits are all 1s (u128::MAX), it's empty
        // free_slot_bits is at offset 72 (8+32+32)
        let free_bits = u128::from_le_bytes(data[72..88].try_into().unwrap_or([0; 16]));
        
        // Also check positions (base_free_lots and quote_free_lots should be 0)
        // Offset for position struct is after is_free (88+16 = 104)
        // base_free_lots is at offset 104, quote_free_lots at 112
        let base_free = i64::from_le_bytes(data[104..112].try_into().unwrap_or([0; 8]));
        let quote_free = i64::from_le_bytes(data[112..120].try_into().unwrap_or([0; 8]));

        free_bits == u128::MAX && base_free == 0 && quote_free == 0
    }
}
