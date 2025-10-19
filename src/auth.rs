use axum::http::{header::AUTHORIZATION, HeaderMap};

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing Authorization header")]
    Missing,
    #[error("invalid Authorization header format")]
    Invalid,
    #[error("unauthorized")]
    Unauthorized,
}

pub fn ensure_authorized(headers: &HeaderMap, config: &Config) -> Result<(), AuthError> {
    let expected = config.api_key();
    if expected.is_empty() {
        return Ok(());
    }

    let header_value = headers
        .get(AUTHORIZATION)
        .ok_or(AuthError::Missing)?
        .to_str()
        .map_err(|_| AuthError::Invalid)?;

    let provided = header_value
        .strip_prefix("Bearer ")
        .or_else(|| header_value.strip_prefix("bearer "))
        .ok_or(AuthError::Invalid)?
        .trim();

    if subtle_equals(provided.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(AuthError::Unauthorized)
    }
}

fn subtle_equals(lhs: &[u8], rhs: &[u8]) -> bool {
    use subtle::ConstantTimeEq;

    if lhs.len() != rhs.len() {
        return false;
    }
    lhs.ct_eq(rhs).into()
}
