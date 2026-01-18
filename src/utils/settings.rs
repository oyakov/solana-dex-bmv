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
    pub large_order_threshold_sol: Decimal,
    pub front_run_tick_size_sol: Decimal,
}

impl Default for OrderGridSettings {
    fn default() -> Self {
        Self {
            orders_per_side: 16,
            buy_volume_multiplier: Decimal::new(12, 1), // 1.2
            sell_volume_multiplier: Decimal::ONE,       // 1.0 (equal)
            rebalance_threshold_percent: Decimal::ONE,  // 1.0%
            large_order_threshold_sol: Decimal::from(50), // 50 SOL
            front_run_tick_size_sol: Decimal::new(1, 6), // 0.000001 SOL
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
            market_id_rent_sol: Decimal::new(4, 1),      // 0.4 SOL
            account_rent_sol: Decimal::new(23, 3),       // 0.023 SOL
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
            usdc_wallet_3: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashVolumeSettings {
    pub enabled: bool,
    pub size_sol: Decimal,
    pub interval_min: u32,
    pub tip_sol: Decimal,
}

impl Default for FlashVolumeSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            size_sol: Decimal::new(1, 1), // 0.1 SOL
            interval_min: 15,
            tip_sol: Decimal::new(1, 3), // 0.001 SOL
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialManagerSettings {
    pub min_sol_reserve_percent: Decimal,
    pub upper_usdc_ratio_max_percent: Decimal,
    pub lower_usdc_ratio_max_percent: Decimal,
    pub min_conversion_barrier_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RugCheckSettings {
    pub enabled: bool,
    pub check_interval_secs: u64,
}

impl Default for RugCheckSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 300, // 5 minutes
        }
    }
}

impl Default for FinancialManagerSettings {
    fn default() -> Self {
        Self {
            min_sol_reserve_percent: Decimal::new(70, 0),
            upper_usdc_ratio_max_percent: Decimal::new(30, 0),
            lower_usdc_ratio_max_percent: Decimal::new(30, 0),
            min_conversion_barrier_usd: Decimal::new(50, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetControlSettings {
    pub total_emission: Decimal,
    pub locked_tokens: Decimal,
}

impl Default for TargetControlSettings {
    fn default() -> Self {
        Self {
            total_emission: Decimal::from(10_000_000),
            locked_tokens: Decimal::from(5_000_000),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillSwitchSettings {
    pub mode: String,
    pub file_path: String,
    pub redis_url: String,
    pub redis_key: String,
}

impl Default for KillSwitchSettings {
    fn default() -> Self {
        Self {
            mode: "file".to_string(),
            file_path: "/tmp/solana-dex-bmv.kill".to_string(),
            redis_url: "redis://127.0.0.1/".to_string(),
            redis_key: "bmv:kill_switch".to_string(),
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
    pub flash_volume: FlashVolumeSettings,
    pub financial_manager: FinancialManagerSettings,
    pub rugcheck: RugCheckSettings,
    pub target_control: TargetControlSettings,
    pub sol_usdc_market_id: String,
    #[serde(default)]
    pub kill_switch: KillSwitchSettings,
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
            .field("flash_volume", &self.flash_volume)
            .field("financial_manager", &self.financial_manager)
            .field("target_control", &self.target_control)
            .field("kill_switch", &self.kill_switch)
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
    300 // 5 minutes to reduce RPC load
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
            flash_volume: FlashVolumeSettings::default(),
            financial_manager: FinancialManagerSettings::default(),
            rugcheck: RugCheckSettings::default(),
            target_control: TargetControlSettings::default(),
            sol_usdc_market_id: "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string(),
            kill_switch: KillSwitchSettings::default(),
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

        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());
        let config_dir = Path::new("config");

        // Load base config
        let base_path = config_dir.join("base.yaml");
        let mut config_value: serde_json::Value = if base_path.exists() {
            let content = fs::read_to_string(base_path)?;
            serde_yaml::from_str(&content)?
        } else {
            // Fallback to legacy config.yaml if it exists
            let legacy_path = Path::new("config.yaml");
            if legacy_path.exists() {
                let content = fs::read_to_string(legacy_path)?;
                serde_yaml::from_str(&content)?
            } else {
                serde_json::to_value(Self::default())?
            }
        };

        // Load profile config and merge
        let profile_path = config_dir.join(format!("{}.yaml", env));
        if profile_path.exists() {
            let content = fs::read_to_string(profile_path)?;
            let profile_value: serde_json::Value = serde_yaml::from_str(&content)?;
            merge_json_values(&mut config_value, profile_value);
        }

        let mut settings: Self = serde_json::from_value(config_value)?;

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

        if let Some(usdc_wallet_3_env) = std::env::var("USDC_WALLET_3")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.wallets.usdc_wallet_3 = usdc_wallet_3_env;
        }

        if let Some(token_mint_env) = std::env::var("TOKEN_MINT")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.token_mint = token_mint_env;
        }

        if let Some(market_id_env) = std::env::var("OPENBOOK_MARKET_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.openbook_market_id = market_id_env;
        }

        if let Some(rpc_primary_http) = std::env::var("RPC_PRIMARY_HTTP")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.rpc_endpoints.primary_http = rpc_primary_http;
        }

        if let Some(rpc_primary_ws) = std::env::var("RPC_PRIMARY_WS")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.rpc_endpoints.primary_ws = rpc_primary_ws;
        }

        if let Some(jito_url) = std::env::var("JITO_BUNDLER_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.jito_bundle.bundler_url = jito_url;
        }

        if let Some(dry_run_env) = std::env::var("DRY_RUN_ENABLED")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.dry_run.enabled = dry_run_env.parse().unwrap_or(settings.dry_run.enabled);
        }

        if let Some(url) = std::env::var("DATABASE_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.database.url = url;
        }

        if let Some(sol_usdc_id) = std::env::var("SOL_USDC_MARKET_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            settings.sol_usdc_market_id = sol_usdc_id;
        }

        Ok(settings)
    }
}

fn merge_json_values(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (serde_json::Value::Object(ref mut a), serde_json::Value::Object(b)) => {
            for (k, v) in b {
                merge_json_values(a.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (a, b) => *a = b,
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
  large_order_threshold_sol: 50.0
  front_run_tick_size_sol: 0.000001
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
flash_volume:
  enabled: true
  size_sol: 0.1
  interval_min: 15
  tip_sol: 0.001
financial_manager:
  min_sol_reserve_percent: 70.0
  upper_usdc_ratio_max_percent: 30.0
  lower_usdc_ratio_max_percent: 30.0
  min_conversion_barrier_usd: 50.0
rugcheck:
  enabled: true
  check_interval_secs: 300
target_control:
  total_emission: 10000000.0
  locked_tokens: 5000000.0
sol_usdc_market_id: "SOL_USDC"
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

    #[test]
    fn test_merge_json_values() {
        use serde_json::json;
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": 3
            }
        });
        let override_val = json!({
            "b": {
                "c": 4,
                "e": 5
            },
            "f": 6
        });
        merge_json_values(&mut base, override_val);
        assert_eq!(base, json!({
            "a": 1,
            "b": {
                "c": 4,
                "d": 3,
                "e": 5
            },
            "f": 6
        }));
    }
}
