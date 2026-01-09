use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::path::Path;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(path: &Path) -> Result<Self> {
        let database_url = format!("sqlite:{}", path.to_string_lossy());

        // Ensure the database file exists or let sqlx create it
        if !path.exists() {
            std::fs::File::create(path)?;
        }

        let pool = SqlitePool::connect(&database_url).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bot_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn set_state(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO bot_state (key, value, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_state(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM bot_state WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.0))
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
