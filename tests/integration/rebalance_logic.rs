use mockall::predicate::*;
use rust_decimal_macros::dec;
use solana_dex_bmv::infra::mocks::{MockDatabaseProvider, MockSolanaProvider};
use solana_dex_bmv::services::FinancialManager;
use solana_dex_bmv::utils::BotSettings;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn test_rebalance_trigger_sol_to_usdc() {
    let mut mock_solana = MockSolanaProvider::new();
    let settings = BotSettings::default();

    // Setup: Price is high (SELL zone), need to swap SOL -> USDC
    let current_price = dec!(160.0);
    let pivot = dec!(150.0);
    let sell_bound = dec!(170.0);
    let buy_bound = dec!(130.0);

    // Expect balance checks
    mock_solana
        .expect_get_balance()
        .returning(|_| Ok(10_000_000_000)); // 10 SOL
    mock_solana
        .expect_get_token_balance()
        .returning(|_, _| Ok(1000_000_000)); // 1000 USDC

    // Expect Jupiter swap call
    mock_solana
        .expect_jupiter_swap()
        .with(
            always(), // signer
            always(), // input_mint (SOL)
            always(), // output_mint (USDC)
            any(),    // amount
            eq(50),   // slippage
        )
        .returning(|_, _, _, _, _| Ok("swap_sig_123".to_string()));

    let solana: Arc<dyn solana_dex_bmv::infra::SolanaProvider> = Arc::new(mock_solana);

    let wallet_manager = Arc::new(
        solana_dex_bmv::infra::WalletManager::new(&[]).expect("Failed to init wallet manager"),
    );

    let financial_manager = FinancialManager::new(settings, solana, wallet_manager);

    let result = financial_manager.rebalance_fiat(current_price, pivot).await;
    assert!(result.is_ok());
}
