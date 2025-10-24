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

    persist_time_series_file(snapshot, dir, limit).await?;

    Ok(file)
}

async fn read_history(file: &PathBuf) -> Result<SnapshotHistory, StorageError> {
    let content = fs::read_to_string(file).await?;
    let history: SnapshotHistory = serde_json::from_str(&content)?;
    Ok(history)
}

async fn persist_time_series_file(
    snapshot: &SystemSnapshot,
    base_dir: &std::path::Path,
    limit: usize,
) -> Result<(), StorageError> {
    let time_series_dir = base_dir.join("time_series");
    fs::create_dir_all(&time_series_dir).await?;

    let file_name = format!("system_snapshot_{}.json", snapshot.timestamp);
    let file_path = time_series_dir.join(file_name);

    let snapshot_json = serde_json::to_string_pretty(snapshot)?;
    fs::write(&file_path, snapshot_json).await?;

    if limit == 0 {
        return Ok(());
    }

    let mut entries = fs::read_dir(&time_series_dir).await?;
    let mut files: Vec<(i64, PathBuf)> = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        let Some(ts_str) = stem.strip_prefix("system_snapshot_") else {
            continue;
        };

        match ts_str.parse::<i64>() {
            Ok(ts) => files.push((ts, path)),
            Err(_) => continue,
        }
    }

    if files.len() <= limit {
        return Ok(());
    }

    files.sort_by_key(|(ts, _)| *ts);
    let prune_count = files.len().saturating_sub(limit);

    for (_, path) in files.into_iter().take(prune_count) {
        let _ = fs::remove_file(path).await;
    }

    Ok(())
}
