use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use solana_dex_bmv::domain::{OrderSide, Trade};
use solana_dex_bmv::services::{GridBuilder, PivotEngine};
use std::time::Instant;

#[tokio::test]
async fn bench_pivot_engine_compute() {
    let engine = PivotEngine::new(
        dec!(100.0),
        7,
        60,
        dec!(1000000),
        dec!(0.02),
        dec!(0.01),
        dec!(0.001),
        dec!(10),
    );

    // 1. Warm up and seed trades
    let trade_count = 10_000;
    println!("Seeding {} trades for performance test...", trade_count);
    for i in 0..trade_count {
        engine
            .record_trade(Trade {
                id: format!("bench-{}", i),
                timestamp: 123456789 + (i as i64),
                price: dec!(100) + (dec!(0.01) * Decimal::from(i % 100)),
                volume: dec!(1.0),
                side: if i % 2 == 0 {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                },
                wallet: "bench".to_string(),
            })
            .await;
    }

    // 2. Measure compute_pivot
    let start = Instant::now();
    let pivot = engine.compute_pivot(&[], &[], None, 0).await;
    let duration = start.elapsed();

    println!("PivotEngine Compute Performance:");
    println!("  Trade Count: {}", trade_count);
    println!("  Duration: {:?}", duration);
    println!("  Pivot: {}", pivot);

    assert!(
        duration.as_millis() < 50,
        "Pivot computation took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn bench_grid_builder() {
    let builder = GridBuilder {
        orders_per_side: 100, // Large grid
        ..Default::default()
    };

    let start = Instant::now();
    let grid: Vec<solana_dex_bmv::domain::GridLevel> = builder.build(dec!(150.0), dec!(10.0)).await;
    let duration = start.elapsed();

    println!("GridBuilder Performance:");
    println!("  Levels: {}", grid.len());
    println!("  Duration: {:?}", duration);

    assert!(
        duration.as_millis() < 10,
        "Grid building took too long: {:?}",
        duration
    );
}
