use crate::domain::{MarketUpdate, Orderbook};
use crate::infra::openbook::{
    parse_book_side_v1, parse_book_side_v2, MarketStateV1, MarketStateV2, OPENBOOK_V2_PROGRAM_ID,
};
use rust_decimal::Decimal;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
// use solana_sdk::hash::Hash; (unused since full path is used below)
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use anyhow::{anyhow, Result};
use base64::Engine;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

pub struct SolanaClient {
    client: RpcClient,
}

#[async_trait::async_trait]
impl crate::infra::SolanaProvider for SolanaClient {
    async fn get_market_data(&self, market_id: &str) -> Result<MarketUpdate> {
        self.get_market_data_impl(market_id).await
    }

    async fn cancel_all_orders(
        &self,
        market_id: &str,
        wallet: &Keypair,
        jito_url: &str,
        tip_lamports: u64,
    ) -> Result<String> {
        self.cancel_all_orders_impl(market_id, wallet, jito_url, tip_lamports)
            .await
    }

    async fn find_open_orders(
        &self,
        market_id: &str,
        owner: &solana_sdk::pubkey::Pubkey,
    ) -> Result<Option<solana_sdk::pubkey::Pubkey>> {
        self.find_open_orders_impl(market_id, owner).await
    }

    async fn health(&self) -> bool {
        self.health_impl().await
    }

    async fn get_orderbook(&self, market_id: &str) -> Result<crate::domain::Orderbook> {
        self.get_orderbook_impl(market_id).await
    }

    async fn get_balance(&self, address: &str) -> Result<u64> {
        self.get_balance_impl(address).await
    }

    async fn get_token_balance(&self, wallet: &Pubkey, mint: &Pubkey) -> Result<u64> {
        self.get_token_balance_impl(wallet, mint).await
    }

    async fn send_bundle(&self, txs: Vec<String>, jito_url: &str) -> Result<String> {
        self.send_bundle_impl(txs, jito_url).await
    }

    async fn jupiter_swap(
        &self,
        signer: &Keypair,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_lamports: u64,
        slippage_bps: u16,
    ) -> Result<String> {
        self.jupiter_swap_impl(
            signer,
            input_mint,
            output_mint,
            amount_lamports,
            slippage_bps,
        )
        .await
    }

    async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        self.get_latest_blockhash_impl().await
    }

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
    ) -> Result<String> {
        self.place_order_impl(
            market_id,
            signer,
            side,
            price,
            size_lots,
            jito_api_url,
            tip_lamports,
            base_wallet,
            quote_wallet,
        )
        .await
    }

    async fn cancel_order(
        &self,
        market_id: &str,
        signer: &Keypair,
        side: u8,
        order_id: u128,
        jito_api_url: &str,
        tip_lamports: u64,
    ) -> Result<String> {
        self.cancel_order_impl(
            market_id,
            signer,
            side,
            order_id,
            jito_api_url,
            tip_lamports,
        )
        .await
    }

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
    ) -> Result<String> {
        self.place_and_cancel_bundle_impl(
            market_id,
            signer,
            place_side,
            place_price,
            place_size,
            cancel_side,
            cancel_order_id,
            jito_api_url,
            tip_lamports,
            base_wallet,
            quote_wallet,
        )
        .await
    }

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
    ) -> Result<String> {
        self.send_flash_volume_bundle_impl(
            market_id,
            wallet_a,
            wallet_b,
            price_lots,
            size_lots,
            tip_lamports,
            jito_url,
            base_mint,
            quote_mint,
        )
        .await
    }

    async fn close_open_orders_account(
        &self,
        signer: &Keypair,
        open_orders: &Pubkey,
    ) -> Result<String> {
        self.close_open_orders_account_impl(signer, open_orders)
            .await
    }

    async fn get_token_largest_accounts(&self, mint: &Pubkey) -> Result<Vec<(Pubkey, u64)>> {
        self.get_token_largest_accounts_impl(mint).await
    }

    async fn get_token_supply(&self, mint: &Pubkey) -> Result<u64> {
        self.get_token_supply_impl(mint).await
    }
}

impl SolanaClient {
    pub fn new(rpc_url: &str, commitment: CommitmentConfig) -> Self {
        Self {
            client: RpcClient::new_with_commitment(rpc_url.to_string(), commitment),
        }
    }

    pub async fn get_balance_impl(&self, owner: &str) -> Result<u64> {
        let pubkey = Pubkey::from_str(owner).map_err(|e| anyhow!("Invalid pubkey: {}", e))?;
        let balance = self.client.get_balance(&pubkey).await?;
        Ok(balance)
    }

    pub async fn get_token_balance_impl(&self, owner: &Pubkey, mint: &Pubkey) -> Result<u64> {
        let ata = spl_associated_token_account::get_associated_token_address(owner, mint);
        match self.client.get_token_account_balance(&ata).await {
            Ok(balance) => {
                let amount = balance.amount.parse::<u64>().unwrap_or(0);
                Ok(amount)
            }
            Err(e) => {
                // If account doesn't exist, balance is 0
                warn!(?e, owner = %owner, mint = %mint, "Failed to get token balance (likely account missing)");
                Ok(0)
            }
        }
    }

    pub async fn get_orderbook_impl(&self, market_id: &str) -> Result<Orderbook> {
        let market_pubkey = Pubkey::from_str(market_id)
            .map_err(|e| anyhow!("Failed to parse market_id '{}': {}", market_id, e))?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;

        let (_bids_pubkey, _asks_pubkey, bids, asks) = if market_data.len() == 388 {
            let market_state = MarketStateV1::unpack(&market_data)?;
            let bids_pubkey = market_state.bids;
            let asks_pubkey = market_state.asks;

            let mut accounts = self
                .client
                .get_multiple_accounts(&[bids_pubkey, asks_pubkey])
                .await?;

            let asks_account = accounts
                .pop()
                .ok_or_else(|| anyhow!("Missing asks account"))?;
            let bids_account = accounts
                .pop()
                .ok_or_else(|| anyhow!("Missing bids account"))?;

            let bids_data = bids_account.map(|a| a.data).unwrap_or_default();
            let asks_data = asks_account.map(|a| a.data).unwrap_or_default();

            let bids = parse_book_side_v1(
                &bids_data,
                true,
                market_state.base_decimals,
                market_state.quote_decimals,
                market_state.base_lot_size,
                market_state.quote_lot_size,
            )?;
            let asks = parse_book_side_v1(
                &asks_data,
                false,
                market_state.base_decimals,
                market_state.quote_decimals,
                market_state.base_lot_size,
                market_state.quote_lot_size,
            )?;
            (bids_pubkey, asks_pubkey, bids, asks)
        } else {
            let market_state = MarketStateV2::unpack(&market_data)?;
            let bids_pubkey = market_state.bids;
            let asks_pubkey = market_state.asks;

            let mut accounts = self
                .client
                .get_multiple_accounts(&[bids_pubkey, asks_pubkey])
                .await?;

            let asks_account = accounts
                .pop()
                .ok_or_else(|| anyhow!("Missing asks account"))?;
            let bids_account = accounts
                .pop()
                .ok_or_else(|| anyhow!("Missing bids account"))?;

            let bids_data = bids_account.map(|a| a.data).unwrap_or_default();
            let asks_data = asks_account.map(|a| a.data).unwrap_or_default();

            let bids = parse_book_side_v2(
                &bids_data,
                true,
                market_state.base_decimals,
                market_state.quote_decimals,
                market_state.base_lot_size,
                market_state.quote_lot_size,
            )?;
            let asks = parse_book_side_v2(
                &asks_data,
                false,
                market_state.base_decimals,
                market_state.quote_decimals,
                market_state.base_lot_size,
                market_state.quote_lot_size,
            )?;
            (bids_pubkey, asks_pubkey, bids, asks)
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        Ok(Orderbook {
            market_id: market_id.to_string(),
            timestamp: now,
            bids,
            asks,
        })
    }

    pub async fn get_market_data_impl(&self, market_id: &str) -> Result<MarketUpdate> {
        let ob = self.get_orderbook_impl(market_id).await?;
        let mid_price = ob
            .get_mid_price()
            .ok_or_else(|| anyhow!("Orderbook is empty, cannot compute mid price"))?;

        Ok(MarketUpdate {
            timestamp: ob.timestamp,
            price: mid_price,
            volume_24h: Decimal::from(5000),
        })
    }

    pub async fn health_impl(&self) -> bool {
        match self.client.get_version().await {
            Ok(_) => true,
            Err(e) => {
                error!(error = ?e, "health_check_failed");
                false
            }
        }
    }

    pub async fn send_bundle_impl(
        &self,
        transactions: Vec<String>,
        jito_api_url: &str,
    ) -> Result<String> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [transactions]
        });

        info!(
            count = transactions.len(),
            api = jito_api_url,
            "sending_jito_bundle"
        );

        let client = reqwest::Client::new();
        let response = client.post(jito_api_url).json(&payload).send().await?;
        let result: serde_json::Value = response.json().await?;

        if let Some(error) = result.get("error") {
            error!(?error, "jito_bundle_error");
            return Err(anyhow!("Jito error: {:?}", error));
        }

        let bundle_id = result["result"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing bundle ID in response"))?
            .to_string();

        info!(?bundle_id, "jito_bundle_sent");
        Ok(bundle_id)
    }

    pub async fn find_open_orders_impl(
        &self,
        market_id: &str,
        owner: &Pubkey,
    ) -> Result<Option<Pubkey>> {
        let program_id = Pubkey::from_str(OPENBOOK_V2_PROGRAM_ID).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse program_id '{}': {}",
                OPENBOOK_V2_PROGRAM_ID,
                e
            )
        })?;
        let market_pubkey = Pubkey::from_str(market_id).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse market_id '{}' in find_open_orders: {}",
                market_id,
                e
            )
        })?;

        let filters = vec![
            solana_client::rpc_filter::RpcFilterType::Memcmp(
                solana_client::rpc_filter::Memcmp::new_raw_bytes(8, owner.to_bytes().to_vec()),
            ),
            solana_client::rpc_filter::RpcFilterType::Memcmp(
                solana_client::rpc_filter::Memcmp::new_raw_bytes(
                    40,
                    market_pubkey.to_bytes().to_vec(),
                ),
            ),
        ];

        let config = solana_client::rpc_config::RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: solana_client::rpc_config::RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                commitment: Some(self.client.commitment()),
                data_slice: None,
                min_context_slot: None,
            },
            with_context: Some(false),
        };

        let accounts = self
            .client
            .get_program_accounts_with_config(&program_id, config)
            .await?;
        Ok(accounts.first().map(|(p, _)| *p))
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn place_order_impl(
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
    ) -> Result<String> {
        let market_pubkey = Pubkey::from_str(market_id)?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;
        let market_state = MarketStateV2::unpack(&market_data)?;

        let open_orders = self
            .find_open_orders_impl(market_id, &signer.pubkey())
            .await?
            .ok_or_else(|| anyhow!("OpenOrders account not found for market {}", market_id))?;

        let user_token_account = if side == 0 { quote_wallet } else { base_wallet };

        let order_ix = crate::infra::openbook::create_place_order_v2_instruction(
            &market_pubkey,
            &open_orders,
            &market_state.asks,
            &market_state.bids,
            &market_state.event_heap,
            &market_state.market_base_vault,
            &market_state.market_quote_vault,
            &signer.pubkey(),
            user_token_account,
            side,
            price,
            size_lots,
            size_lots * price,
            0,
        );

        let tip_ix =
            crate::infra::openbook::create_jito_tip_instruction(&signer.pubkey(), tip_lamports);

        let bh = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[order_ix, tip_ix],
            Some(&signer.pubkey()),
            &[signer as &dyn Signer],
            bh,
        );

        let tx_bytes = bincode::serialize(&tx)?;
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        self.send_bundle_impl(vec![tx_base64], jito_api_url).await
    }

    pub async fn cancel_order_impl(
        &self,
        _market_id: &str,
        signer: &Keypair,
        _side: u8,
        _order_id: u128,
        jito_api_url: &str,
        tip_lamports: u64,
    ) -> Result<String> {
        let tip_ix =
            crate::infra::openbook::create_jito_tip_instruction(&signer.pubkey(), tip_lamports);
        let bh = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[tip_ix],
            Some(&signer.pubkey()),
            &[signer as &dyn Signer],
            bh,
        );
        let tx_bytes = bincode::serialize(&tx)?;
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        self.send_bundle_impl(vec![tx_base64], jito_api_url).await
    }

    #[allow(dead_code)]
    pub async fn cancel_all_orders_impl(
        &self,
        market_id: &str,
        signer: &Keypair,
        jito_api_url: &str,
        tip_lamports: u64,
    ) -> Result<String> {
        let market_pubkey = Pubkey::from_str(market_id)?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;
        let market_state = MarketStateV2::unpack(&market_data)?;

        let open_orders = match self
            .find_open_orders_impl(market_id, &signer.pubkey())
            .await?
        {
            Some(oo) => oo,
            None => {
                info!("No open orders account found to cancel");
                return Ok("no_open_orders".to_string());
            }
        };

        let cancel_bid_ix = crate::infra::openbook::create_cancel_order_v2_instruction(
            &market_pubkey,
            &market_state.bids,
            &market_state.asks,
            &open_orders,
            &signer.pubkey(),
            0,
            u128::MAX, // Use a very large ID or 0 if protocol supports "Cancel All" via specific ID
        );

        let cancel_ask_ix = crate::infra::openbook::create_cancel_order_v2_instruction(
            &market_pubkey,
            &market_state.bids,
            &market_state.asks,
            &open_orders,
            &signer.pubkey(),
            1,
            u128::MAX,
        );

        let tip_ix =
            crate::infra::openbook::create_jito_tip_instruction(&signer.pubkey(), tip_lamports);
        let bh = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[cancel_bid_ix, cancel_ask_ix, tip_ix],
            Some(&signer.pubkey()),
            &[signer as &dyn Signer],
            bh,
        );

        let tx_bytes = bincode::serialize(&tx)?;
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        self.send_bundle_impl(vec![tx_base64], jito_api_url).await
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn place_and_cancel_bundle_impl(
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
    ) -> Result<String> {
        let market_pubkey = Pubkey::from_str(market_id)?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;
        let market_state = MarketStateV2::unpack(&market_data)?;

        let open_orders = self
            .find_open_orders_impl(market_id, &signer.pubkey())
            .await?
            .ok_or_else(|| anyhow!("OpenOrders account not found for market {}", market_id))?;

        let user_token_account = if place_side == 0 {
            quote_wallet
        } else {
            base_wallet
        };
        let place_ix = crate::infra::openbook::create_place_order_v2_instruction(
            &market_pubkey,
            &open_orders,
            &market_state.asks,
            &market_state.bids,
            &market_state.event_heap,
            &market_state.market_base_vault,
            &market_state.market_quote_vault,
            &(*signer).pubkey(),
            user_token_account,
            place_side,
            place_price as i64,
            place_size as i64,
            (place_size * place_price) as i64,
            0,
        );

        let cancel_ix = crate::infra::openbook::create_cancel_order_v2_instruction(
            &market_pubkey,
            &market_state.bids,
            &market_state.asks,
            &open_orders,
            &signer.pubkey(),
            cancel_side,
            cancel_order_id,
        );

        let tip_ix =
            crate::infra::openbook::create_jito_tip_instruction(&signer.pubkey(), tip_lamports);

        let bh = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[place_ix, cancel_ix, tip_ix],
            Some(&signer.pubkey()),
            &[signer as &dyn Signer],
            bh,
        );

        let tx_bytes = bincode::serialize(&tx)?;
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        self.send_bundle_impl(vec![tx_base64], jito_api_url).await
    }
    pub async fn jupiter_swap_impl(
        &self,
        signer: &Keypair,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_lamports: u64,
        slippage_bps: u16,
    ) -> Result<String> {
        let client = reqwest::Client::new();

        // 1. Get Quote
        let quote_url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            input_mint, output_mint, amount_lamports, slippage_bps
        );
        let quote_resp = client.get(&quote_url).send().await?;
        let quote: serde_json::Value = quote_resp.json().await?;

        if quote.get("error").is_some() {
            return Err(anyhow!("Jupiter quote error: {:?}", quote["error"]));
        }

        // 2. Get Swap Transaction
        let swap_url = "https://quote-api.jup.ag/v6/swap";
        let swap_payload = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": signer.pubkey().to_string(),
            "wrapAndUnwrapSol": true
        });

        let swap_resp = client.post(swap_url).json(&swap_payload).send().await?;
        let swap_result: serde_json::Value = swap_resp.json().await?;

        let tx_base64 = swap_result["swapTransaction"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing swapTransaction in Jupiter response"))?;

        // Sign and Send
        let tx_bytes = base64::engine::general_purpose::STANDARD.decode(tx_base64)?;
        let tx: solana_sdk::transaction::VersionedTransaction = bincode::deserialize(&tx_bytes)?;

        let signed_tx =
            solana_sdk::transaction::VersionedTransaction::try_new(tx.message, &[signer])?;

        let sig = self.client.send_and_confirm_transaction(&signed_tx).await?;
        Ok(sig.to_string())
    }

    pub async fn get_latest_blockhash_impl(&self) -> Result<solana_sdk::hash::Hash> {
        self.client.get_latest_blockhash().await.map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn send_flash_volume_bundle_impl(
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
    ) -> Result<String> {
        let market_pubkey = Pubkey::from_str(market_id)?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;
        let market_state = MarketStateV2::unpack(&market_data)?;

        let open_orders_a = self
            .find_open_orders_impl(market_id, &wallet_a.pubkey())
            .await?
            .ok_or_else(|| anyhow!("OpenOrders A not found"))?;
        let open_orders_b = self
            .find_open_orders_impl(market_id, &wallet_b.pubkey())
            .await?
            .ok_or_else(|| anyhow!("OpenOrders B not found"))?;

        let ata_a = spl_associated_token_account::get_associated_token_address(
            &wallet_a.pubkey(),
            quote_mint,
        );
        let ata_b = spl_associated_token_account::get_associated_token_address(
            &wallet_b.pubkey(),
            base_mint,
        );

        // A buys, B sells
        let place_ix_a = crate::infra::openbook::create_place_order_v2_instruction(
            &market_pubkey,
            &open_orders_a,
            &market_state.asks,
            &market_state.bids,
            &market_state.event_heap,
            &market_state.market_base_vault,
            &market_state.market_quote_vault,
            &wallet_a.pubkey(),
            &ata_a,
            0, // Buy
            price_lots,
            size_lots,
            size_lots * price_lots,
            0,
        );

        let place_ix_b = crate::infra::openbook::create_place_order_v2_instruction(
            &market_pubkey,
            &open_orders_b,
            &market_state.asks,
            &market_state.bids,
            &market_state.event_heap,
            &market_state.market_base_vault,
            &market_state.market_quote_vault,
            &wallet_b.pubkey(),
            &ata_b,
            1, // Sell
            price_lots,
            size_lots,
            size_lots * price_lots,
            0,
        );

        let tip_ix =
            crate::infra::openbook::create_jito_tip_instruction(&wallet_a.pubkey(), tip_lamports);

        let bh = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[place_ix_a, place_ix_b, tip_ix],
            Some(&wallet_a.pubkey()),
            &[wallet_a as &dyn Signer, wallet_b as &dyn Signer],
            bh,
        );

        let tx_bytes = bincode::serialize(&tx)?;
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        self.send_bundle_impl(vec![tx_base64], jito_url).await
    }

    pub async fn close_open_orders_account_impl(
        &self,
        signer: &Keypair,
        open_orders: &Pubkey,
    ) -> Result<String> {
        let ix = crate::infra::openbook::create_close_open_orders_v2_instruction(
            open_orders,
            &signer.pubkey(),
            &signer.pubkey(),
        );
        let blockhash = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&signer.pubkey()),
            &[signer],
            blockhash,
        );
        let signature = self.client.send_and_confirm_transaction(&tx).await?;
        Ok(signature.to_string())
    }

    pub async fn get_token_largest_accounts_impl(
        &self,
        mint: &Pubkey,
    ) -> Result<Vec<(Pubkey, u64)>> {
        let resp = self.client.get_token_largest_accounts(mint).await?;
        let mut accounts = Vec::new();
        for acc in resp {
            if let Ok(pubkey) = Pubkey::from_str(&acc.address) {
                let amount = acc.amount.amount.parse::<u64>().unwrap_or(0);
                accounts.push((pubkey, amount));
            }
        }
        Ok(accounts)
    }

    pub async fn get_token_supply_impl(&self, mint: &Pubkey) -> Result<u64> {
        let resp = self.client.get_token_supply(mint).await?;
        let amount = resp.amount.parse::<u64>().unwrap_or(0);
        Ok(amount)
    }
}
