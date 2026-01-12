use mockall::predicate::*;
use rust_decimal_macros::dec;
use solana_dex_bmv::domain::MarketUpdate;
use solana_dex_bmv::infra::mocks::{MockDatabaseProvider, MockSolanaProvider};
use solana_dex_bmv::utils::BotSettings;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

pub struct TestHarness {
    pub solana: Arc<MockSolanaProvider>,
    pub database: Arc<MockDatabaseProvider>,
    pub settings: BotSettings,
}

impl TestHarness {
    pub fn new() -> Self {
        let mut mock_solana = MockSolanaProvider::new();
        let mut mock_database = MockDatabaseProvider::new();
        let settings = BotSettings::default();

        // Default setup for common mocks
        mock_solana.expect_health().returning(|| true);
        mock_database.expect_get_state().returning(|_| Ok(None));
        mock_database
            .expect_get_recent_trades()
            .returning(|_| Ok(vec![]));

        Self {
            solana: Arc::new(mock_solana),
            database: Arc::new(mock_database),
            settings,
        }
    }
}
