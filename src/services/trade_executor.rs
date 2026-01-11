use crate::infra::SolanaProvider;
use crate::utils::BotSettings;
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;

use std::sync::Arc;
use tracing::info;

pub struct TradeExecutor {
    settings: BotSettings,
    solana: Arc<dyn SolanaProvider>,
}

impl TradeExecutor {
    pub fn new(settings: BotSettings, solana: Arc<dyn SolanaProvider>) -> Self {
        Self { settings, solana }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn place_and_cancel_bundle(
        &self,
        market_id: &str,
        signer: &solana_sdk::signature::Keypair,
        place_side: u8,
        place_price: u64,
        place_size: u64,
        cancel_side: u8,
        cancel_order_id: u128,
        base_wallet: &Pubkey,
        quote_wallet: &Pubkey,
    ) -> Result<String> {
        if !self.settings.jito_bundle.enabled {
            return Err(anyhow!(
                "Jito bundle disabled; cannot execute atomic place+cancel"
            ));
        }

        info!(market_id = market_id, "sending_atomic_place_cancel_bundle");

        self.solana
            .place_and_cancel_bundle(
                market_id,
                signer,
                place_side,
                place_price,
                place_size,
                cancel_side,
                cancel_order_id,
                &self.settings.jito_bundle.bundler_url,
                self.settings.jito_bundle.tip_lamports,
                base_wallet,
                quote_wallet,
            )
            .await
    }
}
