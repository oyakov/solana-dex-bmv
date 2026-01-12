use mockall::predicate::*;
use rust_decimal_macros::dec;
use solana_dex_bmv::infra::mocks::MockSolanaProvider;
use solana_dex_bmv::services::FlashVolumeModule;
use solana_dex_bmv::utils::BotSettings;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

#[tokio::test]
async fn test_flash_volume_bundle_submission() {
    let mut mock_solana = MockSolanaProvider::new();
    let mut settings = BotSettings::default();
    settings.flash_volume.base_lot_size = 1000;
    settings.flash_volume.quote_lot_size = 10;

    // Expect bundle submission
    mock_solana
        .expect_send_flash_volume_bundle()
        .with(
            eq(settings.openbook_market_id.clone()),
            always(),    // wallet_a
            always(),    // wallet_b
            any(),       // price_lots
            any(),       // size_lots
            eq(100_000), // tip
            eq(settings.jito.api_url.clone()),
            always(), // base_mint
            always(), // quote_mint
        )
        .returning(|_, _, _, _, _, _, _, _, _| Ok("bundle_sig_456".to_string()));

    let solana: Arc<dyn solana_dex_bmv::infra::SolanaProvider> = Arc::new(mock_solana);

    // Need at least 2 wallets
    let wallet_manager = Arc::new(
        solana_dex_bmv::infra::WalletManager::new(&[
            Keypair::new().to_base58_string(),
            Keypair::new().to_base58_string(),
        ])
        .expect("Failed to init wallet manager"),
    );

    let flash_volume = FlashVolumeModule::new(settings, solana, wallet_manager);

    let result = flash_volume.execute_cycle(dec!(150.0)).await;
    assert!(result.is_ok());
}
