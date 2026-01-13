use mockall::predicate::*;
use rust_decimal_macros::dec;
use solana_dex_bmv::domain::MarketUpdate;
use solana_dex_bmv::infra::mocks::{MockDatabaseProvider, MockSolanaProvider};
use solana_dex_bmv::services::{PivotEngine, TradingService};
use solana_dex_bmv::utils::BotSettings;
use std::sync::Arc;

#[tokio::test]
async fn test_full_trading_loop_integration() {
    let mut mock_solana = MockSolanaProvider::new();
    let mut mock_database = MockDatabaseProvider::new();

    let settings = BotSettings::default();
    let market_id = settings.openbook_market_id.clone();

    // 1. Setup Mock Expectations
    // Get Market Data (Poll)
    mock_solana
        .expect_get_market_data()
        .with(eq(market_id.clone()))
        .returning(move |_| {
            Ok(MarketUpdate {
                price: dec!(150.0),
                volume_24h: dec!(1000000),
                timestamp: 123456789,
            })
        });

    // Database state/trades
    mock_database.expect_get_state().returning(|_| Ok(None));
    mock_database
        .expect_get_recent_trades()
        .returning(|_| Ok(vec![]));

    // 2. Initialize Services
    let solana: Arc<dyn solana_dex_bmv::infra::SolanaProvider> = Arc::new(mock_solana);
    let database: Arc<dyn solana_dex_bmv::infra::DatabaseProvider> = Arc::new(mock_database);
    let wallet_manager = Arc::new(
        solana_dex_bmv::infra::WalletManager::new(&[]).expect("Failed to init wallet manager"),
    );

    let pivot_engine = Arc::new(PivotEngine::new(
        dec!(149.5),
        7,
        60,
        dec!(1000000),
        dec!(0.02),
        dec!(0.01),
        dec!(0.001),
        dec!(10),
    ));

    let price_aggregator = Arc::new(solana_dex_bmv::infra::PriceAggregator::default());
    let trading_service = TradingService::new(
        settings.clone(),
        solana.clone(),
        database.clone(),
        wallet_manager.clone(),
        pivot_engine.clone(),
        price_aggregator,
    );

    // 3. Execute one tick
    let result = trading_service.tick().await;
    assert!(result.is_ok(), "Tick failed: {:?}", result.err());

    // 4. Verify Pivot Engine state mutation via mock side-effects or internal state
    // (In this simple integration, we just ensure it doesn't crash and returns Ok)
}
