use anyhow::Result;
use dotenvy::dotenv;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGridSettings {
    pub orders_per_side: u32,
    pub buy_volume_multiplier: Decimal,
    pub sell_volume_multiplier: Decimal,
    pub rebalance_threshold_percent: Decimal,
}

impl Default for OrderGridSettings {
    fn default() -> Self {
        Self {
            orders_per_side: 16,
            buy_volume_multiplier: Decimal::new(12, 1), // 1.2
            sell_volume_multiplier: Decimal::ONE,       // 1.0 (equal)
            rebalance_threshold_percent: Decimal::ONE,  // 1.0%
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotVwapSettings {
    pub pivot_price: Decimal,
    pub vwap_price: Decimal,
    pub lookback_minutes: u32,
    pub lookback_days: u32,
    pub nominal_daily_volume: Decimal,
    pub market_id_rent_sol: Decimal,
    pub account_rent_sol: Decimal,
    pub jito_tip_sol: Decimal,
    pub fee_bps: Decimal,
}

impl Default for PivotVwapSettings {
    fn default() -> Self {
        Self {
            pivot_price: Decimal::ZERO,
            vwap_price: Decimal::ZERO,
            lookback_minutes: 0,
            lookback_days: 365,
            nominal_daily_volume: Decimal::new(1000, 0), // Default 1000 SOL/unit
            market_id_rent_sol: Decimal::new(4, 1),       // 0.4 SOL
            account_rent_sol: Decimal::new(23, 3),        // 0.023 SOL
            jito_tip_sol: Decimal::ZERO,
            fee_bps: Decimal::new(25, 0), // 25 bps = 0.25%
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBoundsSettings {
    pub buy_percent: Decimal,
    pub sell_percent: Decimal,
}

impl Default for ChannelBoundsSettings {
    fn default() -> Self {
        Self {
            buy_percent: Decimal::new(15, 2),
            sell_percent: Decimal::new(30, 2),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoBundleSettings {
    pub enabled: bool,
    pub tip_lamports: u64,
    pub max_bundle_txs: u32,
    pub bundler_url: String,
}

impl Default for JitoBundleSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            tip_lamports: 0,
            max_bundle_txs: 5,
            bundler_url: "https://mainnet.block-engine.jito.wtf".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSettings {
    pub primary_http: String,
    pub secondary_http: Vec<String>,
    pub primary_ws: String,
}

impl Default for RpcSettings {
    fn default() -> Self {
        Self {
            primary_http: "https://api.mainnet-beta.solana.com".to_string(),
            secondary_http: Vec::new(),
            primary_ws: "wss://api.mainnet-beta.solana.com".to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MultiWalletSettings {
    pub enabled: bool,
    pub keypairs: Vec<String>,
}

impl std::fmt::Debug for MultiWalletSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let masked_keypairs: Vec<String> = self
            .keypairs
            .iter()
            .map(|_| "***MASKED***".to_string())
            .collect();
        f.debug_struct("MultiWalletSettings")
            .field("enabled", &self.enabled)
            .field("keypairs", &masked_keypairs)
            .finish()
    }
}

impl Default for MultiWalletSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            keypairs: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WalletsSettings {
    pub multi_wallet: MultiWalletSettings,
    pub usdc_wallet_3: String,
}

impl std::fmt::Debug for WalletsSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletsSettings")
            .field("multi_wallet", &self.multi_wallet)
            .field("usdc_wallet_3", &"***MASKED***")
            .finish()
    }
}

impl Default for WalletsSettings {
    fn default() -> Self {
        Self {
            multi_wallet: MultiWalletSettings::default(),
            usdc_wallet_3: "CHANGE_ME".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimitsSettings {
    pub max_position_usd: Decimal,
    pub max_order_usd: Decimal,
    pub max_daily_loss_usd: Decimal,
    pub max_open_orders: u32,
}

impl Default for RiskLimitsSettings {
    fn default() -> Self {
        Self {
            max_position_usd: Decimal::ZERO,
            max_order_usd: Decimal::ZERO,
            max_daily_loss_usd: Decimal::ZERO,
            max_open_orders: 0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DryRunSettings {
    pub enabled: bool,
    pub use_live_rpc: bool,
    pub simulate_fills: bool,
}

impl std::fmt::Debug for DryRunSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DryRunSettings")
            .field("enabled", &self.enabled)
            .field("use_live_rpc", &self.use_live_rpc)
            .field("simulate_fills", &self.simulate_fills)
            .finish()
    }
}

impl Default for DryRunSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            use_live_rpc: false,
            simulate_fills: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            url: "postgres://botuser:botpass@localhost:5432/solana_dex".to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BotSettings {
    pub token_mint: String,
    pub openbook_market_id: String,
    pub order_grid: OrderGridSettings,
    pub pivot_vwap: PivotVwapSettings,
    pub channel_bounds: ChannelBoundsSettings,
    pub jito_bundle: JitoBundleSettings,
    pub rpc_endpoints: RpcSettings,
    pub wallets: WalletsSettings,
    pub risk_limits: RiskLimitsSettings,
    #[serde(default)]
    pub database: DatabaseSettings,
    pub dry_run: DryRunSettings,
    #[serde(default = "default_run_mode")]
    pub run_mode: String,
    #[serde(default = "default_trading_tick_interval")]
    pub trading_tick_interval_seconds: u64,
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_seconds: u64,
}

impl std::fmt::Debug for BotSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotSettings")
            .field("token_mint", &self.token_mint)
            .field("openbook_market_id", &self.openbook_market_id)
            .field("order_grid", &self.order_grid)
            .field("pivot_vwap", &self.pivot_vwap)
            .field("channel_bounds", &self.channel_bounds)
            .field("jito_bundle", &self.jito_bundle)
            .field("rpc_endpoints", &self.rpc_endpoints)
            .field("wallets", &self.wallets)
            .field("risk_limits", &self.risk_limits)
            .field("database", &self.database)
            .field("dry_run", &self.dry_run)
            .field("run_mode", &self.run_mode)
            .field(
                "trading_tick_interval_seconds",
                &self.trading_tick_interval_seconds,
            )
            .field(
                "health_check_interval_seconds",
                &self.health_check_interval_seconds,
            )
            .finish()
    }
}

fn default_run_mode() -> String {
    "paper".to_string()
}

fn default_trading_tick_interval() -> u64 {
    10
}

fn default_health_check_interval() -> u64 {
    60
}

impl Default for BotSettings {
    fn default() -> Self {
        Self {
            token_mint: "CHANGE_ME".to_string(),
            openbook_market_id: "B9coHrCxYv7xmPfSU7Z5VfugDqdTdZqZTpBGBdazq8AC".to_string(),
            order_grid: OrderGridSettings::default(),
            pivot_vwap: PivotVwapSettings::default(),
            channel_bounds: ChannelBoundsSettings::default(),
            jito_bundle: JitoBundleSettings::default(),
            rpc_endpoints: RpcSettings::default(),
            wallets: WalletsSettings::default(),
            risk_limits: RiskLimitsSettings::default(),
            database: DatabaseSettings::default(),
            dry_run: DryRunSettings::default(),
            run_mode: default_run_mode(),
            trading_tick_interval_seconds: default_trading_tick_interval(),
            health_check_interval_seconds: default_health_check_interval(),
        }
    }
}

impl BotSettings {
    pub fn load() -> Result<Self> {
        dotenv().ok();

        let config_path =
            std::env::var("BOT_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
        let mut settings = if Path::new(&config_path).exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_yaml::from_str(&content)?
        } else {
            Self::default()
        };

        // Override with environment variables for secrets
        if let Ok(keypairs_env) = std::env::var("WALLET_KEYPAIRS") {
            let keypairs: Vec<String> = keypairs_env
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !keypairs.is_empty() {
                settings.wallets.multi_wallet.keypairs = keypairs;
            }
        }

        if let Ok(usdc_wallet_3_env) = std::env::var("USDC_WALLET_3") {
            settings.wallets.usdc_wallet_3 = usdc_wallet_3_env;
        }

        if let Ok(token_mint_env) = std::env::var("TOKEN_MINT") {
            settings.token_mint = token_mint_env;
        }

        if let Ok(market_id_env) = std::env::var("OPENBOOK_MARKET_ID") {
            settings.openbook_market_id = market_id_env;
        }

        if let Ok(rpc_primary_http) = std::env::var("RPC_PRIMARY_HTTP") {
            settings.rpc_endpoints.primary_http = rpc_primary_http;
        }

        if let Ok(rpc_primary_ws) = std::env::var("RPC_PRIMARY_WS") {
            settings.rpc_endpoints.primary_ws = rpc_primary_ws;
        }

        if let Ok(jito_url) = std::env::var("JITO_BUNDLER_URL") {
            settings.jito_bundle.bundler_url = jito_url;
        }

        if let Ok(dry_run_env) = std::env::var("DRY_RUN_ENABLED") {
            settings.dry_run.enabled = dry_run_env.parse().unwrap_or(settings.dry_run.enabled);
        }

        if let Ok(url) = std::env::var("DATABASE_URL") {
            settings.database.url = url;
        }

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_settings() {
        let yaml = r#"
token_mint: "TEST_MINT"
openbook_market_id: "TEST_MARKET"
order_grid:
  orders_per_side: 32
  buy_volume_multiplier: 1.2
  sell_volume_multiplier: 1.0
  rebalance_threshold_percent: 1.0
pivot_vwap:
  pivot_price: 100.0
  vwap_price: 100.0
  lookback_minutes: 60
  lookback_days: 365
  nominal_daily_volume: 1000.0
  market_id_rent_sol: 0.4
  account_rent_sol: 0.023
  jito_tip_sol: 0.0
  fee_bps: 25
channel_bounds:
  buy_percent: 0.1
  sell_percent: 0.2
jito_bundle:
  enabled: true
  tip_lamports: 1000
  max_bundle_txs: 10
  bundler_url: "http://test.jito"
rpc_endpoints:
  primary_http: "http://test.rpc"
  secondary_http: []
  primary_ws: "ws://test.rpc"
wallets:
  multi_wallet:
    enabled: false
    keypairs: ["key1"]
  usdc_wallet_3: "WALLET3"
risk_limits:
  max_position_usd: 1000.0
  max_order_usd: 100.0
  max_daily_loss_usd: 50.0
  max_open_orders: 5
dry_run:
  enabled: false
  use_live_rpc: true
  simulate_fills: false
"#;
        let settings: BotSettings = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(settings.token_mint, "TEST_MINT");
        assert_eq!(settings.order_grid.orders_per_side, 32);
        assert!(settings.jito_bundle.enabled);
        assert_eq!(settings.rpc_endpoints.primary_http, "http://test.rpc");
        assert!(!settings.wallets.multi_wallet.enabled);
        assert_eq!(settings.wallets.multi_wallet.keypairs[0], "key1");
        assert!(!settings.dry_run.enabled);
    }

    #[test]
    fn test_settings_masking() {
        let mut settings = BotSettings::default();
        settings.wallets.multi_wallet.keypairs = vec!["secret1".to_string()];
        settings.wallets.usdc_wallet_3 = "secret2".to_string();

        let debug_output = format!("{:?}", settings);
        assert!(!debug_output.contains("secret1"));
        assert!(!debug_output.contains("secret2"));
        assert!(debug_output.contains("***MASKED***"));
    }
}
