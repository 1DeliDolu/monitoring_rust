use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{collector::SystemSnapshot, config::Config};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotHistory {
    pub snapshots: Vec<SystemSnapshot>,
}

impl Default for SnapshotHistory {
    fn default() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }
}

pub async fn save_snapshot(
    snapshot: &SystemSnapshot,
    config: &Config,
) -> Result<PathBuf, StorageError> {
    let dir = config.snapshot_dir();
    fs::create_dir_all(dir).await?;

    let file = dir.join("system_snapshot.json");

    // Mevcut snapshot history'yi oku
    let mut history = read_history(&file).await.unwrap_or_default();

    // Yeni snapshot'ı ekle
    history.snapshots.push(snapshot.clone());

    // History limit'e göre eski verileri sil
    let limit = config.history_limit();
    if history.snapshots.len() > limit {
        let skip_count = history.snapshots.len() - limit;
        history.snapshots = history.snapshots.into_iter().skip(skip_count).collect();
    }

    // JSON'a yaz
    let payload = serde_json::to_string_pretty(&history)?;
    fs::write(&file, payload).await?;

    Ok(file)
}

async fn read_history(file: &PathBuf) -> Result<SnapshotHistory, StorageError> {
    let content = fs::read_to_string(file).await?;
    let history: SnapshotHistory = serde_json::from_str(&content)?;
    Ok(history)
}
