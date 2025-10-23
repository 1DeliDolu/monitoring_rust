use axum::http::{header::AUTHORIZATION, HeaderMap};
use std::collections::HashMap;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("unauthorized")]
    Unauthorized,
}

// Query parameter'dan API key kontrolü için fonksiyon
pub fn ensure_authorized_with_query(
    headers: &HeaderMap,
    query_params: &HashMap<String, String>,
    config: &Config,
) -> Result<(), AuthError> {
    let expected = config.api_key();
    if expected.is_empty() {
        return Ok(());
    }

    // Önce header'dan kontrol et
    if let Some(header_value) = headers.get(AUTHORIZATION) {
        if let Ok(header_str) = header_value.to_str() {
            if let Some(provided) = header_str
                .strip_prefix("Bearer ")
                .or_else(|| header_str.strip_prefix("bearer "))
            {
                let provided = provided.trim();
                if subtle_equals(provided.as_bytes(), expected.as_bytes()) {
                    return Ok(());
                }
            }
        }
    }

    // Query parameter'dan kontrol et
    if let Some(api_token) = query_params
        .get("api_token")
        .or_else(|| query_params.get("apitoken"))
        .or_else(|| query_params.get("token"))
        .or_else(|| query_params.get("key"))
    {
        if subtle_equals(api_token.as_bytes(), expected.as_bytes()) {
            return Ok(());
        }
    }

    Err(AuthError::Unauthorized)
}

fn subtle_equals(lhs: &[u8], rhs: &[u8]) -> bool {
    use subtle::ConstantTimeEq;

    if lhs.len() != rhs.len() {
        return false;
    }
    lhs.ct_eq(rhs).into()
}
