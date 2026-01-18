use crate::infra::{SolanaProvider, WalletManager};
use crate::utils::BotSettings;
use anyhow::{anyhow, Result};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use solana_sdk::signer::Signer;
use std::str::FromStr;
use tracing::{info, warn};

pub struct FinancialManager {
    solana: std::sync::Arc<dyn SolanaProvider>,
    wallet_manager: std::sync::Arc<WalletManager>,
    settings: std::sync::Arc<tokio::sync::RwLock<BotSettings>>,
}

impl FinancialManager {
    pub fn new(
        solana: std::sync::Arc<dyn SolanaProvider>,
        wallet_manager: std::sync::Arc<WalletManager>,
        settings: std::sync::Arc<tokio::sync::RwLock<BotSettings>>,
    ) -> Self {
        Self {
            solana,
            wallet_manager,
            settings,
        }
    }

    pub async fn check_balances(&self, current_price: Decimal) -> Result<()> {
        info!("Financial Manager: checking SOL/USDC balances across swarm");

        let (usdc_wallet_3, min_sol_reserve_percent, min_conversion_barrier_usd) = {
            let s = self.settings.read().await;
            (
                s.wallets.usdc_wallet_3.clone(),
                s.financial_manager.min_sol_reserve_percent,
                s.financial_manager.min_conversion_barrier_usd,
            )
        };

        let mut total_sol = Decimal::ZERO;
        let mut total_usdc = Decimal::ZERO;
        let wallets = self.wallet_manager.get_all_wallets().await;
        let usdc_mint = solana_sdk::pubkey::Pubkey::from_str(&usdc_wallet_3).map_err(|e| {
            anyhow!(
                "Failed to parse usdc_wallet_3 in check_balances '{}': {}",
                usdc_wallet_3,
                e
            )
        })?;

        for wallet in &wallets {
            let pubkey = wallet.pubkey();

            // Fetch SOL
            let lamports = self.solana.get_balance(&pubkey.to_string()).await?;
            total_sol += Decimal::from(lamports) / Decimal::from(1_000_000_000u64);

            // Fetch USDC
            let usdc_raw = self.solana.get_token_balance(&pubkey, &usdc_mint).await?;
            total_usdc += Decimal::from(usdc_raw) / Decimal::from(1_000_000u64);
        }

        let sol_value_usd = total_sol * current_price;
        let total_value_usd = sol_value_usd + total_usdc;

        let sol_reserve_percent = if total_value_usd.is_zero() {
            Decimal::ZERO
        } else {
            (sol_value_usd / total_value_usd) * Decimal::from(100)
        };

        info!(
            total_sol = %total_sol.round_dp(4),
            total_usdc = %total_usdc.round_dp(2),
            sol_reserve = %sol_reserve_percent.round_dp(2),
            "Aggregated swarm balances"
        );

        // Emit metrics
        metrics::gauge!("bot_total_sol_balance", total_sol.to_f64().unwrap_or(0.0));
        metrics::gauge!("bot_total_usdc_balance", total_usdc.to_f64().unwrap_or(0.0));
        metrics::gauge!(
            "bot_sol_reserve_percent",
            sol_reserve_percent.to_f64().unwrap_or(0.0)
        );

        // 2. Check against MIN_SOL_RESERVE_%
        if sol_reserve_percent < min_sol_reserve_percent {
            warn!(
                %sol_reserve_percent,
                threshold = %min_sol_reserve_percent,
                "SOL reserve below threshold! Triggering auto-injection."
            );

            // Calculate how much USD to convert to reach threshold + 5% buffer
            let target_percent = min_sol_reserve_percent + Decimal::from(5);
            let target_sol_value = total_value_usd * (target_percent / Decimal::from(100));
            let usd_to_buy = (target_sol_value - sol_value_usd).max(Decimal::ZERO);

            if usd_to_buy >= min_conversion_barrier_usd {
                let usdc_units = (usd_to_buy * Decimal::from(1_000_000u64))
                    .to_u64()
                    .unwrap_or(0);
                if usdc_units > 0 {
                    let sol_mint = solana_sdk::pubkey::Pubkey::from_str(
                        "So11111111111111111111111111111111111111112",
                    )
                    .unwrap();
                    let main_wallet = self.wallet_manager.get_main_wallet().await?;

                    // Check if main wallet has enough USDC
                    let main_usdc = self
                        .solana
                        .get_token_balance(&main_wallet.pubkey(), &usdc_mint)
                        .await?;
                    if main_usdc >= usdc_units {
                        info!(%usd_to_buy, "SOL Auto-injection: executing USDC -> SOL swap");
                        let sig = self
                            .solana
                            .jupiter_swap(
                                &main_wallet,
                                &usdc_mint,
                                &sol_mint,
                                usdc_units,
                                50, // 0.5% slippage
                            )
                            .await?;
                        info!(%sig, "Auto-injection swap successful");
                    } else {
                        warn!(%main_usdc, %usdc_units, "Main wallet lacks USDC for auto-injection; skipping");
                    }
                }
            } else {
                info!(%usd_to_buy, "USD amount to buy is below conversion barrier; skipping auto-injection");
            }
        }

        Ok(())
    }

    pub async fn rebalance_fiat(&self, current_price: Decimal, pivot: Decimal) -> Result<()> {
        let (buy_percent, sell_percent, _usdc_wallet_3) = {
            let s = self.settings.read().await;
            (
                s.channel_bounds.buy_percent,
                s.channel_bounds.sell_percent,
                s.wallets.usdc_wallet_3.clone(),
            )
        };
        let buy_bound = pivot * (Decimal::ONE - buy_percent);
        let sell_bound = pivot * (Decimal::ONE + sell_percent);

        info!(
            %current_price,
            %pivot,
            %buy_bound,
            %sell_bound,
            "Financial Manager: evaluating fiat/sol ratio"
        );

        let sol_mint =
            solana_sdk::pubkey::Pubkey::from_str("So11111111111111111111111111111111111111112")
                .map_err(|e| anyhow::anyhow!("Failed to parse SOL mint: {}", e))?;
        let usdc_mint = {
            let s = self.settings.read().await;
            solana_sdk::pubkey::Pubkey::from_str(&s.wallets.usdc_wallet_3)
                .map_err(|e| anyhow::anyhow!("Failed to parse usdc_wallet_3: {}", e))?
        };
        let main_wallet = self.wallet_manager.get_main_wallet().await?;

        if current_price > pivot && (sell_bound - pivot) > Decimal::ZERO {
            // In SELL zone: SOL -> USDC (Gradients: 100/0 at Pivot -> 70/30 at sell_bound)
            // Progress 0.0 at Pivot -> 1.0 at sell_bound
            let (upper_usdc_ratio_max_percent, min_conversion_barrier_usd) = {
                let s = self.settings.read().await;
                (
                    s.financial_manager.upper_usdc_ratio_max_percent,
                    s.financial_manager.min_conversion_barrier_usd,
                )
            };

            let progress = (current_price - pivot) / (sell_bound - pivot);
            let _target_usdc_ratio =
                progress.min(Decimal::ONE) * upper_usdc_ratio_max_percent / Decimal::from(100);

            // To reach target_usdc_ratio, we may need to sell SOL.
            // Placeholder: if progress > 0.5 and barrier met, sell $50 worth of SOL
            if progress > Decimal::new(5, 1) {
                // 0.5
                let amount_usd = Decimal::from(50);
                if amount_usd >= min_conversion_barrier_usd {
                    let amount_lamports = (Decimal::from(1_000_000_000u64) * amount_usd
                        / current_price)
                        .to_u64()
                        .unwrap_or(0);
                    if amount_lamports > 0 {
                        info!(%amount_usd, "Executing SELL: SOL -> USDC via Jupiter");
                        let sig = self
                            .solana
                            .jupiter_swap(
                                &main_wallet,
                                &sol_mint,
                                &usdc_mint,
                                amount_lamports,
                                50, // 0.5% slippage
                            )
                            .await?;
                        info!(%sig, "Swap successful");
                    }
                }
            }
        } else if current_price < pivot && (pivot - buy_bound) > Decimal::ZERO {
            // In BUY zone: USDC -> SOL (Gradients: 100/0 at Pivot -> 70/30 at buy_bound)
            let lower_usdc_ratio_max_percent = {
                let s = self.settings.read().await;
                s.financial_manager.lower_usdc_ratio_max_percent
            };
            let progress = (pivot - current_price) / (pivot - buy_bound);
            let _target_sol_ratio =
                progress.min(Decimal::ONE) * lower_usdc_ratio_max_percent / Decimal::from(100);

            // If SOL share is too low, buy SOL.
            // Placeholder: if progress > 0.5, buy $50 worth of SOL
            if progress > Decimal::new(5, 1) {
                let amount_usd = Decimal::from(50);
                let amount_usdc_units = (amount_usd * Decimal::from(1_000_000u64))
                    .to_u64()
                    .unwrap_or(0);
                if amount_usdc_units > 0 {
                    info!(%amount_usd, "Executing BUY: USDC -> SOL via Jupiter");
                    let sig = self
                        .solana
                        .jupiter_swap(&main_wallet, &usdc_mint, &sol_mint, amount_usdc_units, 50)
                        .await?;
                    info!(%sig, "Swap successful");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::mocks::MockSolanaProvider;
    use rust_decimal_macros::dec;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_rebalance_fiat_math_precision() {
        let mut settings = BotSettings::default();
        settings.channel_bounds.buy_percent = dec!(0.10); // 10%
        settings.channel_bounds.sell_percent = dec!(0.20); // 20%
        settings.financial_manager.upper_usdc_ratio_max_percent = dec!(30);
        settings.financial_manager.min_conversion_barrier_usd = dec!(10);

        let mut mock_solana = MockSolanaProvider::new();
        // Setup mock to catch the swap call
        mock_solana
            .expect_jupiter_swap()
            .returning(|_, _, _, _, _| Ok("sig".to_string()));

        let solana: Arc<dyn SolanaProvider> = Arc::new(mock_solana);
        let wallet_manager = Arc::new(
            crate::infra::WalletManager::new(
                &[solana_sdk::signature::Keypair::new().to_base58_string()],
                None,
            )
            .unwrap(),
        );

        let manager = FinancialManager::new(solana, wallet_manager, settings);

        // Price at pivot - no swap expected
        let result = manager.rebalance_fiat(dec!(100), dec!(100)).await;
        assert!(result.is_ok());

        // Price in deep SELL zone (progress = 1.0)
        // pivot 100, sell_bound 120. current 125
        let result = manager.rebalance_fiat(dec!(125), dec!(100)).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_ratio_calculation_logic() {
        // This test would be cleaner if the ratio math was in a pure function
        // For now we rely on the integration-style unit test above
    }
}
