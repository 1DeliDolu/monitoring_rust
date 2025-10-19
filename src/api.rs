use std::sync::Arc;

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
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Unauthorized => (axum::http::StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
            ApiError::BadRequest(_) => (axum::http::StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            ApiError::Internal => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
                .into_response(),
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::Missing | AuthError::Invalid | AuthError::Unauthorized => ApiError::Unauthorized,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

pub async fn system(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<SystemSnapshot>, ApiError> {
    authorise(&state, &headers)?;
    let snapshot = state.latest_snapshot().await;
    Ok(Json(snapshot))
}

pub async fn history(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<SystemSnapshot>>, ApiError> {
    authorise(&state, &headers)?;

    let max = state.config().history_limit();
    let limit = query.limit.unwrap_or(max).min(max);
    let snapshots = state.history(limit).await;
    Ok(Json(snapshots))
}

pub async fn apps(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise(&state, &headers)?;
    Ok(Json(json!({ "apps": [] })))
}

pub async fn tasks(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise(&state, &headers)?;
    Ok(Json(json!({ "tasks": [] })))
}

pub async fn webtest(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise(&state, &headers)?;
    Ok(Json(json!({ "tests": [] })))
}

pub async fn alerts(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorise(&state, &headers)?;
    Ok(Json(json!({ "alerts": [] })))
}

fn authorise(state: &Arc<AppState>, headers: &HeaderMap) -> Result<(), ApiError> {
    auth::ensure_authorized(headers, state.config()).map_err(ApiError::from)
}
