use std::path::PathBuf;

use tokio::fs;

use crate::{collector::SystemSnapshot, config::Config};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

pub async fn save_snapshot(
    snapshot: &SystemSnapshot,
    config: &Config,
) -> Result<PathBuf, StorageError> {
    let dir = config.snapshot_dir();
    fs::create_dir_all(dir).await?;

    let file = dir.join(format!("system_snapshot_{}.json", snapshot.timestamp));
    let payload = serde_json::to_string_pretty(snapshot)?;
    fs::write(&file, payload).await?;

    Ok(file)
}
