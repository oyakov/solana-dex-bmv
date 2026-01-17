use anyhow::{anyhow, Result};
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;
use std::sync::Arc;
use tokio::sync::RwLock;

use solana_sdk::bs58;
use tracing::{debug, error, info, warn};

pub struct WalletManager {
    wallets: RwLock<Vec<Arc<Keypair>>>,
    database: Option<Arc<dyn crate::infra::DatabaseProvider>>,
}

impl WalletManager {
    pub fn new(
        wallet_secrets: &[String],
        database: Option<Arc<dyn crate::infra::DatabaseProvider>>,
    ) -> Result<Self> {
        let mut loaded_wallets = Vec::new();

        // Load from secrets (env/config)
        for secret in wallet_secrets {
            if let Ok(kp) = Self::parse_secret(secret) {
                loaded_wallets.push(Arc::new(kp));
            }
        }

        let manager = Self {
            wallets: RwLock::new(loaded_wallets),
            database,
        };

        Ok(manager)
    }

    pub async fn load_from_db(&self) -> Result<()> {
        if let Some(db) = &self.database {
            info!("Loading wallets from database...");
            let db_wallets = db.get_wallets().await?;
            info!(count = db_wallets.len(), "Retrieved wallets from database");
            let mut wallets = self.wallets.write().await;
            for (pubkey, secret) in db_wallets {
                match Self::parse_secret(&secret) {
                    Ok(kp) => {
                        if !wallets.iter().any(|w| w.pubkey().to_string() == pubkey) {
                            info!(%pubkey, "Loaded wallet from database");
                            wallets.push(Arc::new(kp));
                        } else {
                            debug!(%pubkey, "Wallet already loaded, skipping");
                        }
                    }
                    Err(e) => {
                        error!(%pubkey, error = ?e, "Failed to parse wallet secret from database");
                    }
                }
            }
        } else {
            warn!("No database provider configured in WalletManager, skipping DB load");
        }
        Ok(())
    }

    fn parse_secret(secret: &str) -> Result<Keypair> {
        // Try as file path first
        if std::path::Path::new(secret).exists() {
            if let Ok(kp) = read_keypair_file(secret) {
                return Ok(kp);
            }
        }

        // Try as base58 string
        let bytes = bs58::decode(secret)
            .into_vec()
            .map_err(|e| anyhow!("Invalid base58: {}", e))?;
        Keypair::from_bytes(&bytes).map_err(|e| anyhow!("Invalid keypair bytes: {}", e))
    }

    pub async fn add_wallet(&self, secret: &str, persist: bool) -> Result<String> {
        let kp = Self::parse_secret(secret)?;
        let pubkey = kp.pubkey().to_string();

        let mut wallets = self.wallets.write().await;
        // Check if already exists
        if wallets.iter().any(|w| w.pubkey().to_string() == pubkey) {
            return Err(anyhow!("Wallet already exists in manager"));
        }

        if persist {
            if let Some(db) = &self.database {
                db.save_wallet(&pubkey, secret).await?;
            }
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
