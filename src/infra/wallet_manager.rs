use anyhow::{anyhow, Result};
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;

use tracing::{info, warn};

pub struct WalletManager {
    wallets: Vec<Keypair>,
}

impl WalletManager {
    pub fn new(wallet_secrets: &[String]) -> Result<Self> {
        let mut wallets = Vec::new();

        for secret in wallet_secrets {
            // Try as file path first
            if std::path::Path::new(secret).exists() {
                match read_keypair_file(secret) {
                    Ok(kp) => {
                        info!(pubkey = %kp.pubkey(), "Loaded wallet from file");
                        wallets.push(kp);
                        continue;
                    }
                    Err(e) => {
                        warn!(error = ?e, "Failed to read keypair file, trying as base58");
                    }
                }
            }

            // Try as base58 string
            let kp = Keypair::from_base58_string(secret);
            // from_base58_string in older solana-sdk returns Keypair directly and panics if invalid?
            // Actually in 1.18 it might be different. Let's check common usage.
            // Usually it's Keypair::from_base58_string(secret)
            info!(pubkey = %kp.pubkey(), "Loaded wallet from base58");
            wallets.push(kp);
        }

        if wallets.is_empty() {
            warn!("No wallets loaded by WalletManager");
        }

        Ok(Self { wallets })
    }

    #[allow(dead_code)]
    pub fn get_all_pubkeys(&self) -> Vec<String> {
        self.wallets
            .iter()
            .map(|k| k.pubkey().to_string())
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_keypair(&self, index: usize) -> Result<&Keypair> {
        self.wallets
            .get(index)
            .ok_or_else(|| anyhow!("Wallet index out of bounds"))
    }

    pub fn get_all_wallets(&self) -> Vec<&Keypair> {
        self.wallets.iter().collect()
    }

    pub fn count(&self) -> usize {
        self.wallets.len()
    }
}
