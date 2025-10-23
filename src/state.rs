use std::{collections::VecDeque, sync::Arc};

use tokio::sync::{Mutex, RwLock};

use crate::{collector::SystemSnapshot, config::Config};

pub type SharedState = Arc<AppState>;

pub struct AppState {
    config: Config,
    latest_snapshot: RwLock<SystemSnapshot>,
    history: Mutex<VecDeque<SystemSnapshot>>,
}

impl AppState {
    pub fn new(config: Config, initial_snapshot: SystemSnapshot) -> Self {
        let mut history = VecDeque::new();
        history.push_front(initial_snapshot.clone());
        Self {
            config,
            latest_snapshot: RwLock::new(initial_snapshot),
            history: Mutex::new(history),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn latest_snapshot(&self) -> SystemSnapshot {
        self.latest_snapshot.read().await.clone()
    }

    pub async fn record_snapshot(&self, snapshot: SystemSnapshot) {
        {
            let mut writer = self.latest_snapshot.write().await;
            *writer = snapshot.clone();
        }

        let mut history = self.history.lock().await;
        history.push_front(snapshot);
        while history.len() > self.config.history_limit() {
            history.pop_back();
        }
    }

    pub async fn history(&self, limit: usize) -> Vec<SystemSnapshot> {
        let history = self.history.lock().await;
        history.iter().take(limit).cloned().collect()
    }
}
