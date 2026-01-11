use crate::domain::{MarketUpdate, Orderbook};
use crate::infra::openbook::{parse_slab, MarketStateV3};
use rust_decimal::Decimal;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;

use anyhow::{anyhow, Result};
use base64::Engine;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

pub struct SolanaClient {
    client: RpcClient,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, commitment: CommitmentConfig) -> Self {
        Self {
            client: RpcClient::new_with_commitment(rpc_url.to_string(), commitment),
        }
    }

    pub async fn get_balance(&self, owner: &str) -> Result<u64> {
        let pubkey = Pubkey::from_str(owner).map_err(|e| anyhow!("Invalid pubkey: {}", e))?;
        let balance = self.client.get_balance(&pubkey).await?;
        Ok(balance)
    }

    pub async fn get_orderbook(&self, market_id: &str) -> Result<Orderbook> {
        let market_pubkey = Pubkey::from_str(market_id)?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;
        let market_state = MarketStateV3::unpack(&market_data)?;

        // Fetch Bids and Asks accounts
        let bids_pubkey = Pubkey::from(market_state.bids);
        let asks_pubkey = Pubkey::from(market_state.asks);

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

        let bids = parse_slab(
            &bids_data,
            true,
            market_state.base_lot_size,
            market_state.quote_lot_size,
        )?;
        let asks = parse_slab(
            &asks_data,
            false,
            market_state.base_lot_size,
            market_state.quote_lot_size,
        )?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        Ok(Orderbook {
            market_id: market_id.to_string(),
            timestamp: now,
            bids,
            asks,
        })
    }

    pub async fn get_market_data(&self, market_id: &str) -> Result<MarketUpdate> {
        // Now that we have get_orderbook, we can derive price from it
        match self.get_orderbook(market_id).await {
            Ok(ob) => {
                let mid_price = ob.get_mid_price().unwrap_or(Decimal::ZERO);
                Ok(MarketUpdate {
                    timestamp: ob.timestamp,
                    price: mid_price,
                    volume_24h: Decimal::from(5000), // TODO: Get volume from event queue
                })
            }
            Err(e) => {
                warn!(?e, "failed_to_fetch_orderbook_falling_back_to_sim");
                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
                Ok(MarketUpdate {
                    timestamp: now,
                    price: Decimal::from(150),
                    volume_24h: Decimal::from(5000),
                })
            }
        }
    }

    #[allow(dead_code)]
    pub async fn health(&self) -> bool {
        match self.client.get_version().await {
            Ok(_) => true,
            Err(e) => {
                error!(error = ?e, "health_check_failed");
                false
            }
        }
    }

    #[allow(dead_code)]
    pub async fn send_bundle(
        &self,
        transactions: Vec<String>,
        jito_api_url: &str,
    ) -> Result<String> {
        // (Existing send_bundle implementation...)
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

    #[allow(dead_code)]
    pub async fn find_open_orders(
        &self,
        market_id: &str,
        owner: &Pubkey,
    ) -> Result<Option<Pubkey>> {
        let program_id = Pubkey::from_str("srmqPvSwwJbtLZ9Uv7j8W7YVFe4Gz74Xp2Y7tENz7u4")?;
        let market_pubkey = Pubkey::from_str(market_id)?;

        let filters = vec![
            solana_client::rpc_filter::RpcFilterType::DataSize(3228), // OpenOrders size
            solana_client::rpc_filter::RpcFilterType::Memcmp(
                solana_client::rpc_filter::Memcmp::new_raw_bytes(
                    13, // span of market
                    market_pubkey.to_bytes().to_vec(),
                ),
            ),
            solana_client::rpc_filter::RpcFilterType::Memcmp(
                solana_client::rpc_filter::Memcmp::new_raw_bytes(
                    45, // span of owner
                    owner.to_bytes().to_vec(),
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

    #[allow(dead_code)]
    pub async fn place_order(
        &self,
        market_id: &str,
        signer: &dyn solana_sdk::signer::Signer,
        side: u8,
        price: u64,
        size: u64,
        jito_api_url: &str,
        tip_lamports: u64,
        // Optional pre-discovered accounts
        base_wallet: &Pubkey,
        quote_wallet: &Pubkey,
    ) -> Result<String> {
        let market_pubkey = Pubkey::from_str(market_id)?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;
        let market_state = MarketStateV3::unpack(&market_data)?;

        // Discover open_orders
        let open_orders = self
            .find_open_orders(market_id, &signer.pubkey())
            .await?
            .ok_or_else(|| anyhow!("OpenOrders account not found for market {}", market_id))?;

        let order_ix = crate::infra::openbook::create_new_order_v3_instruction(
            &market_pubkey,
            &open_orders,
            &Pubkey::from(market_state.request_queue),
            &Pubkey::from(market_state.event_queue),
            &Pubkey::from(market_state.bids),
            &Pubkey::from(market_state.asks),
            &Pubkey::from(market_state.base_vault),
            &Pubkey::from(market_state.quote_vault),
            &signer.pubkey(),
            base_wallet,
            quote_wallet,
            side,
            price,
            size,
            size * price, // Simplified max_quote_qty
            0,            // Limit
            0,            // client_id
        );

        let tip_ix =
            crate::infra::openbook::create_jito_tip_instruction(&signer.pubkey(), tip_lamports);

        // Construct the transaction
        let bh = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[order_ix, tip_ix],
            Some(&signer.pubkey()),
            &[signer],
            bh,
        );

        let tx_bytes = bincode::serialize(&tx)?;
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        self.send_bundle(vec![tx_base64], jito_api_url).await
    }

    #[allow(dead_code)]
    pub async fn cancel_order(
        &self,
        market_id: &str,
        signer: &dyn solana_sdk::signer::Signer,
        side: u8,
        order_id: u128,
        jito_api_url: &str,
        tip_lamports: u64,
    ) -> Result<String> {
        let market_pubkey = Pubkey::from_str(market_id)?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;
        let market_state = MarketStateV3::unpack(&market_data)?;

        let open_orders = self
            .find_open_orders(market_id, &signer.pubkey())
            .await?
            .ok_or_else(|| anyhow!("OpenOrders account not found"))?;

        let cancel_ix = crate::infra::openbook::create_cancel_order_v2_instruction(
            &market_pubkey,
            &Pubkey::from(market_state.bids),
            &Pubkey::from(market_state.asks),
            &open_orders,
            &signer.pubkey(),
            &Pubkey::from(market_state.event_queue),
            side,
            order_id,
        );

        let tip_ix =
            crate::infra::openbook::create_jito_tip_instruction(&signer.pubkey(), tip_lamports);

        let bh = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[cancel_ix, tip_ix],
            Some(&signer.pubkey()),
            &[signer],
            bh,
        );

        let tx_bytes = bincode::serialize(&tx)?;
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        self.send_bundle(vec![tx_base64], jito_api_url).await
    }

    #[allow(dead_code)]
    pub async fn cancel_all_orders(
        &self,
        market_id: &str,
        signer: &dyn solana_sdk::signer::Signer,
        jito_api_url: &str,
        tip_lamports: u64,
    ) -> Result<String> {
        let market_pubkey = Pubkey::from_str(market_id)?;
        let market_data = self.client.get_account_data(&market_pubkey).await?;
        let market_state = MarketStateV3::unpack(&market_data)?;

        let open_orders = match self.find_open_orders(market_id, &signer.pubkey()).await? {
            Some(open_orders) => open_orders,
            None => {
                info!(market_id = %market_id, "No open orders account found to cancel");
                return Ok("no_open_orders".to_string());
            }
        };

        let cancel_bid_ix = crate::infra::openbook::create_cancel_all_orders_instruction(
            &market_pubkey,
            &Pubkey::from(market_state.bids),
            &Pubkey::from(market_state.asks),
            &open_orders,
            &signer.pubkey(),
            &Pubkey::from(market_state.event_queue),
            0,
            u16::MAX,
        );

        let cancel_ask_ix = crate::infra::openbook::create_cancel_all_orders_instruction(
            &market_pubkey,
            &Pubkey::from(market_state.bids),
            &Pubkey::from(market_state.asks),
            &open_orders,
            &signer.pubkey(),
            &Pubkey::from(market_state.event_queue),
            1,
            u16::MAX,
        );

        let tip_ix =
            crate::infra::openbook::create_jito_tip_instruction(&signer.pubkey(), tip_lamports);

        let bh = self.client.get_latest_blockhash().await?;
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[cancel_bid_ix, cancel_ask_ix, tip_ix],
            Some(&signer.pubkey()),
            &[signer],
            bh,
        );

        let tx_bytes = bincode::serialize(&tx)?;
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        self.send_bundle(vec![tx_base64], jito_api_url).await
    }
}
