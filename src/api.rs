use std::{sync::Arc, collections::HashMap};

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

use crate::{
    auth::{self, AuthError},
    collector::SystemSnapshot,
    state::{AppState, SharedState},
};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("unauthorized")]
    Unauthorized,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (axum::http::StatusCode::UNAUTHORIZED, self.to_string()).into_response()
    }
}

impl From<AuthError> for ApiError {
    fn from(_value: AuthError) -> Self {
        ApiError::Unauthorized
    }
}

pub async fn system(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<SystemSnapshot>, ApiError> {
    authorise_with_query(&state, &headers, &query)?;
    let snapshot = state.latest_snapshot().await;
    Ok(Json(snapshot))
}

#[derive(Debug, Deserialize)]
pub struct SystemQuery {
    pub limit: Option<usize>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    #[serde(flatten)]
    pub auth_params: HashMap<String, String>,
}

pub async fn history(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<SystemQuery>,
) -> Result<Json<Vec<SystemSnapshot>>, ApiError> {
    authorise_with_query(&state, &headers, &query.auth_params)?;

    let max = state.config().history_limit();
    let limit = query.limit.unwrap_or(max).min(max);
    let mut snapshots = state.history(limit).await;
    
    // Zaman aralığına göre filtrele
    if let Some(from) = query.from {
        snapshots.retain(|s| s.timestamp >= from);
    }
    if let Some(to) = query.to {
        snapshots.retain(|s| s.timestamp <= to);
    }
    
    Ok(Json(snapshots))
}

pub async fn apps(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise_with_query(&state, &headers, &query)?;
    Ok(Json(json!({ "apps": [] })))
}

pub async fn tasks(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise_with_query(&state, &headers, &query)?;
    Ok(Json(json!({ "tasks": [] })))
}

pub async fn webtest(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise_with_query(&state, &headers, &query)?;
    Ok(Json(json!({ "tests": [] })))
}

pub async fn alerts(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise_with_query(&state, &headers, &query)?;
    Ok(Json(json!({ "alerts": [] })))
}

pub async fn snapshot_file(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise_with_query(&state, &headers, &query)?;
    
    // JSON dosyasını oku ve döndür
    let file_path = state.config().snapshot_dir().join("system_snapshot.json");
    
    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json_data) => Ok(Json(json_data)),
                Err(_) => Ok(Json(json!({ "snapshots": [] }))),
            }
        }
        Err(_) => Ok(Json(json!({ "snapshots": [] }))),
    }
}

fn authorise_with_query(
    state: &Arc<AppState>, 
    headers: &HeaderMap, 
    query: &HashMap<String, String>
) -> Result<(), ApiError> {
    auth::ensure_authorized_with_query(headers, query, state.config()).map_err(ApiError::from)
}
