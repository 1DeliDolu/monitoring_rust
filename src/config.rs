use std::{
    env,
    path::{Path, PathBuf},
    time::Duration,
};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    api_key: String,
    bind_address: String,
    snapshot_dir: PathBuf,
    history_limit: usize,
    collection_interval: Duration,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("SYSTEM_API_KEY is missing - set it in the environment or .env file")]
    MissingApiKey,
    #[error("invalid COLLECTION_INTERVAL_SECS value: {0}")]
    InvalidInterval(String),
    #[error("invalid HISTORY_LIMIT value: {0}")]
    InvalidHistory(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let api_key = env::var("SYSTEM_API_KEY").map_err(|_| ConfigError::MissingApiKey)?;
        let bind_address =
            env::var("API_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:7000".to_string());
        let snapshot_dir = env::var("SNAPSHOT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/snapshots"));

        let collection_interval =
            env::var("COLLECTION_INTERVAL_SECS").unwrap_or_else(|_| "1".to_string());
        let collection_interval_secs: f64 = collection_interval
            .parse()
            .map_err(|_| ConfigError::InvalidInterval(collection_interval.clone()))?;
        let history_limit = env::var("HISTORY_LIMIT").unwrap_or_else(|_| "288".to_string());
        let history_limit: usize = history_limit
            .parse()
            .map_err(|_| ConfigError::InvalidHistory(history_limit.clone()))?;

        Ok(Self {
            api_key,
            bind_address,
            snapshot_dir,
            history_limit,
            collection_interval: Duration::from_secs_f64(collection_interval_secs.max(1.0)),
        })
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn bind_address(&self) -> &str {
        &self.bind_address
    }

    pub fn snapshot_dir(&self) -> &Path {
        &self.snapshot_dir
    }

    pub fn history_limit(&self) -> usize {
        self.history_limit
    }

    pub fn collection_interval(&self) -> Duration {
        self.collection_interval
    }
}
