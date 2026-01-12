use mockall::predicate::*;
use rust_decimal_macros::dec;
use solana_dex_bmv::infra::mocks::MockSolanaProvider;
use solana_dex_bmv::services::RentRecoveryService;
use solana_dex_bmv::utils::BotSettings;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

#[tokio::test]
async fn test_rent_recovery_scanning() {
    let mut mock_solana = MockSolanaProvider::new();
    let settings = BotSettings::default();

    let wallet = Keypair::new();
    let oo_account = Pubkey::new_unique();

    // Expect find_open_orders
    mock_solana
        .expect_find_open_orders()
        .returning(move |_, _| Ok(Some(oo_account)));

    let solana: Arc<dyn solana_dex_bmv::infra::SolanaProvider> = Arc::new(mock_solana);

    let wallet_manager = Arc::new(
        solana_dex_bmv::infra::WalletManager::new(&[wallet.to_base58_string()])
            .expect("Failed to init wallet manager"),
    );

    let rent_recovery = RentRecoveryService::new(settings, solana, wallet_manager);

    let result = rent_recovery.recover_rent().await;
    assert!(result.is_ok());
}
