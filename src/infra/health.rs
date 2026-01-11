use anyhow::{anyhow, Result};
use metrics::gauge;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use crate::infra::{Database, SolanaClient};
use crate::utils::BotSettings;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Failed,
    Skipped,
}

impl ServiceStatus {
    pub fn to_status_string(&self) -> String {
        match self {
            ServiceStatus::Healthy => "HEALTHY".to_string(),
            ServiceStatus::Degraded => "DEGRADED".to_string(),
            ServiceStatus::Failed => "FAILED".to_string(),
            ServiceStatus::Skipped => "SKIPPED".to_string(),
        }
    }
}

pub struct HealthReport {
    pub service_name: String,
    pub status: ServiceStatus,
    pub latency_ms: u128,
    pub message: Option<String>,
}

pub struct HealthChecker {
    solana: Arc<SolanaClient>,
    database: Arc<Database>,
    settings: BotSettings,
}

impl HealthChecker {
    pub fn new(solana: Arc<SolanaClient>, database: Arc<Database>, settings: BotSettings) -> Self {
        Self {
            solana,
            database,
            settings,
        }
    }

    pub async fn check_solana(&self) -> HealthReport {
        let start = Instant::now();
        let healthy = self.solana.health().await;
        let latency = start.elapsed().as_millis();

        if healthy {
            HealthReport {
                service_name: "Solana RPC".to_string(),
                status: ServiceStatus::Healthy,
                latency_ms: latency,
                message: None,
            }
        } else {
            HealthReport {
                service_name: "Solana RPC".to_string(),
                status: ServiceStatus::Failed,
                latency_ms: latency,
                message: Some("RPC health check failed".to_string()),
            }
        }
    }

    pub async fn check_database(&self) -> HealthReport {
        let start = Instant::now();
        // A simple state check to verify DB is alive
        let result = self.database.get_state("health_check").await;
        let latency = start.elapsed().as_millis();

        match result {
            Ok(_) => HealthReport {
                service_name: "Database (SQLite)".to_string(),
                status: ServiceStatus::Healthy,
                latency_ms: latency,
                message: None,
            },
            Err(e) => HealthReport {
                service_name: "Database (SQLite)".to_string(),
                status: ServiceStatus::Failed,
                latency_ms: latency,
                message: Some(format!("DB check failed: {}", e)),
            },
        }
    }

    pub async fn check_jito(&self) -> HealthReport {
        if !self.settings.jito_bundle.enabled {
            return HealthReport {
                service_name: "Jito Bundler".to_string(),
                status: ServiceStatus::Skipped,
                latency_ms: 0,
                message: Some("Disabled in config".to_string()),
            };
        }

        let start = Instant::now();
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTipAccounts",
            "params": []
        });

        let result = client
            .post(&self.settings.jito_bundle.bundler_url)
            .json(&payload)
            .send()
            .await;
        let latency = start.elapsed().as_millis();

        match result {
            Ok(resp) if resp.status().is_success() => HealthReport {
                service_name: "Jito Bundler".to_string(),
                status: ServiceStatus::Healthy,
                latency_ms: latency,
                message: None,
            },
            Ok(resp) => HealthReport {
                service_name: "Jito Bundler".to_string(),
                status: ServiceStatus::Degraded,
                latency_ms: latency,
                message: Some(format!("Status: {}", resp.status())),
            },
            Err(e) => HealthReport {
                service_name: "Jito Bundler".to_string(),
                status: ServiceStatus::Failed,
                latency_ms: latency,
                message: Some(format!("Connection failed: {}", e)),
            },
        }
    }

    pub async fn check_openbook(&self) -> HealthReport {
        let start = Instant::now();
        let result = self
            .solana
            .get_orderbook(&self.settings.openbook_market_id)
            .await;
        let latency = start.elapsed().as_millis();

        match result {
            Ok(_) => HealthReport {
                service_name: "OpenBook DEX".to_string(),
                status: ServiceStatus::Healthy,
                latency_ms: latency,
                message: None,
            },
            Err(e) => HealthReport {
                service_name: "OpenBook DEX".to_string(),
                status: ServiceStatus::Failed,
                latency_ms: latency,
                message: Some(format!("Market check failed: {}", e)),
            },
        }
    }

    pub async fn run_all_checks(&self) -> Vec<HealthReport> {
        info!("Running connectivity health checks...");

        let mut reports = Vec::new();
        reports.push(self.check_solana().await);
        reports.push(self.check_database().await);
        reports.push(self.check_jito().await);
        reports.push(self.check_openbook().await);

        self.emit_metrics(&reports);

        reports
    }

    pub fn emit_metrics(&self, reports: &[HealthReport]) {
        for report in reports {
            let status_val = match report.status {
                ServiceStatus::Healthy => 1.0,
                ServiceStatus::Degraded => 0.5,
                ServiceStatus::Failed => 0.0,
                ServiceStatus::Skipped => -1.0,
            };

            gauge!(
                "bot_service_health_status",
                status_val,
                "service" => report.service_name.clone()
            );

            if report.status != ServiceStatus::Skipped {
                gauge!(
                    "bot_service_latency_ms",
                    report.latency_ms as f64,
                    "service" => report.service_name.clone()
                );
            }
        }
    }

    pub fn display_reports(reports: &[HealthReport]) {
        println!("\n=== CONNECTIVITY STATUS ===");
        println!(
            "{:<20} | {:<10} | {:<10} | {}",
            "Service", "Status", "Latency", "Notes"
        );
        println!("{}", "-".repeat(70));

        for report in reports {
            let latency_str = if report.status == ServiceStatus::Skipped {
                "-".to_string()
            } else {
                format!("{}ms", report.latency_ms)
            };

            println!(
                "{:<20} | {:<10} | {:<10} | {}",
                report.service_name,
                report.status.to_status_string(),
                latency_str,
                report.message.as_deref().unwrap_or("")
            );
        }
        println!("=========================\n");
    }

    pub async fn verify_critical_services(&self, reports: &[HealthReport]) -> Result<()> {
        for report in reports {
            if (report.service_name == "Solana RPC" || report.service_name == "Database (SQLite)")
                && report.status == ServiceStatus::Failed
            {
                return Err(anyhow!(
                    "Critical service failure: {}. Cannot proceed.",
                    report.service_name
                ));
            }
        }
        Ok(())
    }
}
