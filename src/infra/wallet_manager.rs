use anyhow::{anyhow, Result};
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;
use std::sync::Arc;
use tokio::sync::RwLock;

use tracing::{info, warn};

pub struct WalletManager {
    wallets: RwLock<Vec<Arc<Keypair>>>,
}

impl WalletManager {
    pub fn new(wallet_secrets: &[String]) -> Result<Self> {
        let mut loaded_wallets = Vec::new();

        for secret in wallet_secrets {
            // Try as file path first
            if std::path::Path::new(secret).exists() {
                match read_keypair_file(secret) {
                    Ok(kp) => {
                        info!(pubkey = %kp.pubkey(), "Loaded wallet from file");
                        loaded_wallets.push(Arc::new(kp));
                        continue;
                    }
                    Err(e) => {
                        warn!(error = ?e, "Failed to read keypair file, trying as base58");
                    }
                }
            }

            // Try as base58 string
            match Keypair::from_base58_string(secret).try_into() {
                Ok(kp) => {
                    let kp: Keypair = kp;
                    info!(pubkey = %kp.pubkey(), "Loaded wallet from base58");
                    loaded_wallets.push(Arc::new(kp));
                }
                Err(_) => {
                    warn!("Failed to load wallet from base58 string");
                }
            }
        }

        if loaded_wallets.is_empty() {
            warn!("No wallets loaded by WalletManager");
        }

        Ok(Self {
            wallets: RwLock::new(loaded_wallets),
        })
    }

    pub async fn add_wallet(&self, secret: &str) -> Result<String> {
        // Try as base58 string
        let kp = match Keypair::from_base58_string(secret).try_into() {
            Ok(kp) => kp,
            Err(_) => return Err(anyhow!("Invalid base58 wallet secret")),
        };

        let kp: Keypair = kp;
        let pubkey = kp.pubkey().to_string();

        let mut wallets = self.wallets.write().await;
        // Check if already exists
        if wallets.iter().any(|w| w.pubkey().to_string() == pubkey) {
            return Err(anyhow!("Wallet already exists in manager"));
        }

        info!(%pubkey, "Added new wallet to manager");
        wallets.push(Arc::new(kp));
        Ok(pubkey)
    }

    pub async fn get_all_pubkeys(&self) -> Vec<String> {
        self.wallets
            .read()
            .await
            .iter()
            .map(|k| k.pubkey().to_string())
            .collect()
    }

    pub async fn get_keypair(&self, index: usize) -> Result<Arc<Keypair>> {
        self.wallets
            .read()
            .await
            .get(index)
            .cloned()
            .ok_or_else(|| anyhow!("Wallet index out of bounds"))
    }

    pub async fn get_all_wallets(&self) -> Vec<Arc<Keypair>> {
        self.wallets.read().await.clone()
    }

    pub async fn get_main_wallet(&self) -> Result<Arc<Keypair>> {
        self.wallets
            .read()
            .await
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No wallets available in WalletManager"))
    }

    pub async fn count(&self) -> usize {
        self.wallets.read().await.len()
    }
}
