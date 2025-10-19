mod api;
mod auth;
mod collector;
mod config;
mod scheduler;
mod state;
mod storage;
mod ui;

use crate::config::Config;
use crate::state::SharedState;
use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from `.env` when available.
    let _ = dotenvy::dotenv();

    // Initialise tracing with sensible defaults.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,agent=debug")),
        )
        .with_target(false)
        .compact()
        .init();

    let config = Config::from_env()?;

    // Perform an initial collection so API/UI have data immediately.
    let initial_snapshot = tokio::task::spawn_blocking({
        let interval = config.collection_interval();
        move || collector::collect_snapshot(None, interval)
    })
    .await
    .map_err(|err| {
        error!("collector task panicked: {}", err);
        err
    })?;

    let state: SharedState = Arc::new(state::AppState::new(config.clone(), initial_snapshot));

    scheduler::spawn(state.clone());

    let app = Router::new()
        .route("/api/system", get(api::system))
        .route("/api/history", get(api::history))
        .route("/api/snapshots", get(api::snapshot_file))
        .route("/api/apps", get(api::apps))
        .route("/api/tasks", get(api::tasks))
        .route("/api/webtest", get(api::webtest))
        .route("/api/alerts", get(api::alerts))
        .route("/ui", get(ui::show_ui))
        .with_state(state.clone());

    let bind_addr = config.bind_address().to_string();
    let listener = TcpListener::bind(&bind_addr).await?;

    info!("🚀 API: http://{}/api/system", bind_addr);
    info!("🖥️  UI: http://{}/ui", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
