use crate::domain::{MarketUpdate, Trade};
use anyhow::Result;
use async_trait::async_trait;
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::Keypair;

#[async_trait]
pub trait SolanaProvider: Send + Sync {
    async fn get_market_data(&self, market_id: &str) -> Result<MarketUpdate>;
    async fn cancel_all_orders(
        &self,
        market_id: &str,
        wallet: &Keypair,
        jito_url: &str,
        tip_lamports: u64,
    ) -> Result<String>;
    async fn find_open_orders(&self, market_id: &str, owner: &Pubkey) -> Result<Option<Pubkey>>;
    async fn health(&self) -> bool;
    async fn get_orderbook(&self, market_id: &str) -> Result<crate::domain::Orderbook>;
    async fn get_balance(&self, address: &str) -> Result<u64>;
    async fn get_token_balance(&self, wallet: &Pubkey, mint: &Pubkey) -> Result<u64>;
    async fn send_bundle(&self, txs: Vec<String>, jito_url: &str) -> Result<String>;
    async fn jupiter_swap(
        &self,
        signer: &Keypair,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_lamports: u64,
        slippage_bps: u16,
    ) -> Result<String>;
    async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash>;
    #[allow(clippy::too_many_arguments)]
    async fn place_order(
        &self,
        market_id: &str,
        signer: &Keypair,
        side: u8,
        price: i64,
        size_lots: i64,
        jito_api_url: &str,
        tip_lamports: u64,
        base_wallet: &Pubkey,
        quote_wallet: &Pubkey,
    ) -> Result<String>;
    async fn cancel_order(
        &self,
        market_id: &str,
        signer: &Keypair,
        side: u8,
        order_id: u128,
        jito_api_url: &str,
        tip_lamports: u64,
    ) -> Result<String>;
    #[allow(clippy::too_many_arguments)]
    async fn place_and_cancel_bundle(
        &self,
        market_id: &str,
        signer: &Keypair,
        place_side: u8,
        place_price: u64,
        place_size: u64,
        cancel_side: u8,
        cancel_order_id: u128,
        jito_api_url: &str,
        tip_lamports: u64,
        base_wallet: &Pubkey,
        quote_wallet: &Pubkey,
    ) -> Result<String>;
    #[allow(clippy::too_many_arguments)]
    async fn send_flash_volume_bundle(
        &self,
        market_id: &str,
        wallet_a: &Keypair,
        wallet_b: &Keypair,
        price_lots: i64,
        size_lots: i64,
        tip_lamports: u64,
        jito_url: &str,
        base_mint: &Pubkey,
        quote_mint: &Pubkey,
    ) -> Result<String>;
    async fn close_open_orders_account(
        &self,
        signer: &Keypair,
        open_orders: &Pubkey,
    ) -> Result<String>;
    async fn get_open_orders_account_data(&self, oo_pubkey: &Pubkey) -> Result<Vec<u8>>;
    async fn create_market(
        &self,
        base_mint: &Pubkey,
        quote_mint: &Pubkey,
        market_authority: &solana_sdk::signer::keypair::Keypair,
    ) -> Result<Pubkey>;
    async fn get_token_largest_accounts(&self, mint: &Pubkey) -> Result<Vec<(Pubkey, u64)>>;
    async fn get_token_supply(&self, mint: &Pubkey) -> Result<u64>;
}

#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    async fn get_state(&self, key: &str) -> Result<Option<String>>;
    async fn set_state(&self, key: &str, value: &str) -> Result<()>;
    async fn get_recent_trades(&self, since_ts: i64) -> Result<Vec<Trade>>;
    async fn save_trade(&self, trade: &Trade) -> Result<()>;
    async fn save_price_tick(&self, asset_price: Decimal, sol_price: Decimal) -> Result<()>;
    async fn save_historical_price_ticks(&self, ticks: Vec<(i64, Decimal, Decimal)>) -> Result<()>;
    async fn get_price_history(&self, since_ts: i64) -> Result<Vec<crate::domain::PriceTick>>;
    async fn save_latency_report(&self, report: &crate::infra::health::HealthReport) -> Result<()>;
    async fn get_latency_history(
        &self,
        service_name: &str,
        since_ts: i64,
    ) -> Result<Vec<crate::domain::LatencyTick>>;
    async fn save_wallet(&self, pubkey: &str, secret: &str) -> Result<()>;
    async fn get_wallets(&self) -> Result<Vec<(String, String)>>;
}
