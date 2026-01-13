use anyhow::{anyhow, Result};
use metrics::gauge;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
struct RugCheckReport {
    pub score: i64,
}

pub struct RugCheckService {
    client: Client,
}

impl RugCheckService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_score(&self, mint: &str) -> Result<i64> {
        let url = format!("https://rugcheck.xyz/api/v1/tokens/{}/report", mint);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "RugCheck API failed with status: {}",
                response.status()
            ));
        }

        let report: RugCheckReport = response.json().await?;

        info!(mint, score = report.score, "RugCheck score updated");
        gauge!("bot_rugcheck_score", report.score as f64);

        Ok(report.score)
    }
}
