use anyhow::Result;
use dotenvy::dotenv;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaSettings {
    pub rpc_url: String,
    pub commitment: String,
    pub default_fee_payer: Option<String>,
    pub wallets: Vec<String>, // List of base58 private keys or paths
}

impl Default for SolanaSettings {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            default_fee_payer: None,
            wallets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub path: PathBuf,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            path: PathBuf::from("bot_state.sqlite"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSettings {
    pub timeout_seconds: f64,
    pub user_agent: String,
}

impl Default for HttpSettings {
    fn default() -> Self {
        Self {
            timeout_seconds: 10.0,
            user_agent: "solana-dex-bmv-bot/0.1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySettings {
    pub market_id: String,
    pub pivot_interval_seconds: f64,
    pub grid_spacing_bps: u32,
    pub rebalance_threshold_bps: u32,
    pub orders_per_side: u32,
    pub buy_channel_width: Decimal,
    pub sell_channel_width: Decimal,
    pub lookback_days: u32,
    pub initial_fade_in_days: u32,
}

impl Default for StrategySettings {
    fn default() -> Self {
        Self {
            market_id: "B9coHrCxYv7xmPfSU7Z5VfugDqdTdZqZTpBGBdazq8AC".to_string(), // BMV/SOL OpenBook
            pivot_interval_seconds: 30.0,
            grid_spacing_bps: 25,
            rebalance_threshold_bps: 50,
            orders_per_side: 16,
            buy_channel_width: Decimal::from_str_radix("0.15", 10).unwrap(),
            sell_channel_width: Decimal::from_str_radix("0.30", 10).unwrap(),
            lookback_days: 365,
            initial_fade_in_days: 30,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoSettings {
    pub enabled: bool,
    pub api_url: String,
    pub tip_lamports: u64,
}

impl Default for JitoSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            api_url: "https://mainnet.block-engine.jito.wtf/api/v1/bundles".to_string(),
            tip_lamports: 5_000_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSettings {
    pub solana: SolanaSettings,
    pub database: DatabaseSettings,
    pub http: HttpSettings,
    pub strategy: StrategySettings,
    pub jito: JitoSettings,
    pub run_mode: String, // "paper" or "live"
}

impl Default for BotSettings {
    fn default() -> Self {
        Self {
            solana: SolanaSettings::default(),
            database: DatabaseSettings::default(),
            http: HttpSettings::default(),
            strategy: StrategySettings::default(),
            jito: JitoSettings::default(),
            run_mode: "paper".to_string(),
        }
    }
}

impl BotSettings {
    pub fn load() -> Result<Self> {
        dotenv().ok();
        // Simplified loader for now, in a real case we would use config-rs
        // or manually check env vars.
        Ok(Self::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = BotSettings::default();
        assert_eq!(settings.run_mode, "paper");
        assert_eq!(settings.solana.commitment, "confirmed");
        assert_eq!(settings.strategy.orders_per_side, 16);
        assert!(!settings.jito.enabled);
    }

    #[test]
    fn test_serialization() {
        let settings = BotSettings::default();
        let yaml = serde_yaml::to_string(&settings).unwrap();
        assert!(yaml.contains("run_mode: paper"));
        assert!(yaml.contains("orders_per_side: 16"));
        
        let deserialized: BotSettings = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.run_mode, settings.run_mode);
        assert_eq!(deserialized.strategy.orders_per_side, settings.strategy.orders_per_side);
    }
}
