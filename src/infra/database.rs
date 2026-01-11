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
            "CREATE TABLE IF NOT EXISTS trades (
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

        // Add index on timestamp for faster VWAP queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades (timestamp)")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub async fn set_state(&self, key: &str, value: &str) -> Result<()> {
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

    pub async fn get_state(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM bot_state WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.0))
    }

    pub async fn save_trade(&self, trade: &Trade) -> Result<()> {
        let side_str = match trade.side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        };

        sqlx::query(
            "INSERT INTO trades (id, timestamp, price, volume, side, wallet)
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

    pub async fn get_recent_trades(&self, since_timestamp: i64) -> Result<Vec<Trade>> {
        let rows: Vec<(String, i64, String, String, String, String)> = sqlx::query_as(
            "SELECT id, timestamp, price, volume, side, wallet FROM trades WHERE timestamp >= $1 ORDER BY timestamp ASC",
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

        db.set_state("test_key", "test_value").await?;
        let val = db.get_state("test_key").await?;
        assert_eq!(val, Some("test_value".to_string()));

        db.set_state("test_key", "new_value").await?;
        let val = db.get_state("test_key").await?;
        assert_eq!(val, Some("new_value".to_string()));

        let non_existent = db.get_state("missing").await?;
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

        let trades = db.get_recent_trades(500).await?;
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].id, "trade_1");
        assert_eq!(trades[0].price, Decimal::from_str("1.23456789")?);

        let trades_none = db.get_recent_trades(1500).await?;
        assert_eq!(trades_none.len(), 0);

        db.close().await;
        Ok(())
    }
}
