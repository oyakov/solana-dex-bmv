use crate::utils::KillSwitchSettings;
use anyhow::{Context, Result};
use redis::AsyncCommands;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub enum KillSwitchBackend {
    File { path: PathBuf },
    Redis { url: String, key: String },
}

#[derive(Debug, Clone)]
pub struct KillSwitch {
    backend: KillSwitchBackend,
}

impl KillSwitch {
    pub fn from_settings(settings: &KillSwitchSettings) -> Self {
        match settings.mode.as_str() {
            "redis" => Self {
                backend: KillSwitchBackend::Redis {
                    url: settings.redis_url.clone(),
                    key: settings.redis_key.clone(),
                },
            },
            "file" => Self {
                backend: KillSwitchBackend::File {
                    path: PathBuf::from(settings.file_path.clone()),
                },
            },
            other => {
                warn!(mode = %other, "Unknown kill switch mode; defaulting to file");
                Self {
                    backend: KillSwitchBackend::File {
                        path: PathBuf::from(settings.file_path.clone()),
                    },
                }
            }
        }
    }

    pub async fn is_triggered(&self) -> Result<bool> {
        match &self.backend {
            KillSwitchBackend::File { path } => match fs::metadata(path).await {
                Ok(_) => Ok(true),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(err) => Err(err).context("checking kill switch file"),
            },
            KillSwitchBackend::Redis { url, key } => {
                let client = redis::Client::open(url.as_str())
                    .with_context(|| format!("opening redis {}", url))?;
                let mut conn = client.get_async_connection().await?;
                let value: Option<String> = conn.get(key).await?;
                Ok(value
                    .map(|flag| {
                        let normalized = flag.trim().to_lowercase();
                        !(normalized.is_empty() || normalized == "0" || normalized == "false")
                    })
                    .unwrap_or(false))
            }
        }
    }

    pub async fn trigger(&self, reason: &str) -> Result<()> {
        match &self.backend {
            KillSwitchBackend::File { path } => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).await?;
                }
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let payload = format!("triggered_at={timestamp}\nreason={reason}\n");
                fs::write(path, payload).await?;
                info!(path = %path.display(), "Kill switch file written");
                Ok(())
            }
            KillSwitchBackend::Redis { url, key } => {
                let client = redis::Client::open(url.as_str())
                    .with_context(|| format!("opening redis {}", url))?;
                let mut conn = client.get_async_connection().await?;
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let payload = format!("triggered_at={timestamp} reason={reason}");
                conn.set::<_, _, ()>(key, payload).await?;
                info!(redis_key = %key, "Kill switch redis key set");
                Ok(())
            }
        }
    }
}
