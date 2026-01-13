use crate::domain::{OrderSide, Trade};
use anyhow::Result;
use metrics::counter;
use rust_decimal::Decimal;
use sqlx::postgres::PgPool;
use std::str::FromStr;

#[allow(dead_code)]
pub struct Database {
    pool: PgPool,
}

#[async_trait::async_trait]
impl crate::infra::DatabaseProvider for Database {
    async fn get_state(&self, key: &str) -> Result<Option<String>> {
        self.get_state_impl(key).await
    }

    async fn set_state(&self, key: &str, value: &str) -> Result<()> {
        self.set_state_impl(key, value).await
    }

    async fn get_recent_trades(&self, since_ts: i64) -> Result<Vec<Trade>> {
        self.get_recent_trades_impl(since_ts).await
    }

    async fn save_trade(&self, trade: &Trade) -> Result<()> {
        self.save_trade_impl(trade).await
    }

    async fn save_price_tick(&self, asset_price: Decimal, sol_price: Decimal) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        sqlx::query(
            "INSERT INTO price_history (timestamp, asset_price, sol_price)
            VALUES ($1, $2, $3)
            ON CONFLICT (timestamp) DO NOTHING",
        )
        .bind(timestamp)
        .bind(asset_price.to_string())
        .bind(sol_price.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_historical_price_ticks(&self, ticks: Vec<(i64, Decimal, Decimal)>) -> Result<()> {
        for (ts, asset_price, sol_price) in ticks {
            sqlx::query(
                "INSERT INTO price_history (timestamp, asset_price, sol_price)
                VALUES ($1, $2, $3)
                ON CONFLICT (timestamp) DO NOTHING",
            )
            .bind(ts)
            .bind(asset_price.to_string())
            .bind(sol_price.to_string())
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn get_price_history(&self, since_ts: i64) -> Result<Vec<crate::domain::PriceTick>> {
        let rows: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT timestamp, asset_price, sol_price FROM price_history 
             WHERE timestamp >= $1 ORDER BY timestamp ASC",
        )
        .bind(since_ts)
        .fetch_all(&self.pool)
        .await?;

        let ticks = rows
            .into_iter()
            .map(|r| crate::domain::PriceTick {
                timestamp: r.0,
                asset_price: Decimal::from_str(&r.1).unwrap_or_default(),
                sol_price: Decimal::from_str(&r.2).unwrap_or_default(),
            })
            .collect();

        Ok(ticks)
    }
}

#[allow(dead_code)]
impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;

        // Initialize tables
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bot_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS trades_history (
                id TEXT PRIMARY KEY,
                timestamp BIGINT NOT NULL,
                price TEXT NOT NULL,
                volume TEXT NOT NULL,
                side TEXT NOT NULL,
                wallet TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS price_history (
                timestamp BIGINT PRIMARY KEY,
                asset_price TEXT NOT NULL,
                sol_price TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        // Add index on timestamp for faster VWAP queries
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_trades_history_timestamp ON trades_history (timestamp)",
        )
        .execute(&pool)
        .await?;

        let legacy_table: Option<String> =
            sqlx::query_scalar("SELECT to_regclass('public.trades')::text")
                .fetch_one(&pool)
                .await?;
        if legacy_table.is_some() {
            sqlx::query(
                "INSERT INTO trades_history (id, timestamp, price, volume, side, wallet)
                SELECT id, timestamp, price, volume, side, wallet
                FROM trades
                ON CONFLICT (id) DO NOTHING",
            )
            .execute(&pool)
            .await?;
        }

        Ok(Self { pool })
    }

    pub async fn set_state_impl(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO bot_state (key, value, updated_at)
            VALUES ($1, $2, CURRENT_TIMESTAMP)
            ON CONFLICT(key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = CURRENT_TIMESTAMP",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_state_impl(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM bot_state WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.0))
    }

    pub async fn save_trade_impl(&self, trade: &Trade) -> Result<()> {
        let side_str = match trade.side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        };

        sqlx::query(
            "INSERT INTO trades_history (id, timestamp, price, volume, side, wallet)
            VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&trade.id)
        .bind(trade.timestamp)
        .bind(trade.price.to_string())
        .bind(trade.volume.to_string())
        .bind(side_str)
        .bind(&trade.wallet)
        .execute(&self.pool)
        .await?;

        counter!("bot_trades_saved_total", 1, "side" => side_str);

        Ok(())
    }

    pub async fn get_recent_trades_impl(&self, since_timestamp: i64) -> Result<Vec<Trade>> {
        let rows: Vec<(String, i64, String, String, String, String)> = sqlx::query_as(
            "SELECT id, timestamp, price, volume, side, wallet FROM trades_history WHERE timestamp >= $1 ORDER BY timestamp ASC, id ASC",
        )
        .bind(since_timestamp)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            trades.push(Trade {
                id: row.0,
                timestamp: row.1,
                price: Decimal::from_str(&row.2).unwrap_or_default(),
                volume: Decimal::from_str(&row.3).unwrap_or_default(),
                side: if row.4 == "buy" {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                },
                wallet: row.5,
            });
        }

        Ok(trades)
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::DatabaseProvider;

    async fn get_test_db() -> Option<Database> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        Database::connect(&url).await.ok()
    }

    #[tokio::test]
    async fn test_database_state() -> Result<()> {
        let db = match get_test_db().await {
            Some(db) => db,
            None => return Ok(()), // Skip if no test DB
        };

        db.set_state_impl("test_key", "test_value").await?;
        let val = db.get_state_impl("test_key").await?;
        assert_eq!(val, Some("test_value".to_string()));

        db.set_state_impl("test_key", "new_value").await?;
        let val = db.get_state_impl("test_key").await?;
        assert_eq!(val, Some("new_value".to_string()));

        let non_existent = db.get_state_impl("missing").await?;
        assert!(non_existent.is_none());

        db.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_database_trades() -> Result<()> {
        let db = match get_test_db().await {
            Some(db) => db,
            None => return Ok(()), // Skip if no test DB
        };

        let trade = Trade {
            id: "trade_1".to_string(),
            timestamp: 1000,
            price: Decimal::from_str("1.23456789")?,
            volume: Decimal::from_str("100.0")?,
            side: OrderSide::Buy,
            wallet: "wallet_1".to_string(),
        };

        db.save_trade(&trade).await?;

        let trades = db.get_recent_trades_impl(500).await?;
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].id, "trade_1");
        assert_eq!(trades[0].price, Decimal::from_str("1.23456789")?);

        let trades_none = db.get_recent_trades_impl(1500).await?;
        assert_eq!(trades_none.len(), 0);

        db.close().await;
        Ok(())
    }
}
