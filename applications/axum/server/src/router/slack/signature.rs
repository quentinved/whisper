use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;
use tracing::warn;

use crate::app_state::AppState;

type HmacSha256 = Hmac<Sha256>;

const MAX_TIMESTAMP_AGE_SECONDS: i64 = 300; // 5 minutes

pub async fn verify_slack_signature(
    State(app_state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let signing_secret = app_state.slack_signing_secret().ok_or_else(|| {
        warn!("Slack signing secret not configured");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    let timestamp = request
        .headers()
        .get("X-Slack-Request-Timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing X-Slack-Request-Timestamp header");
            StatusCode::UNAUTHORIZED.into_response()
        })?
        .to_string();

    let signature = request
        .headers()
        .get("X-Slack-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing X-Slack-Signature header");
            StatusCode::UNAUTHORIZED.into_response()
        })?
        .to_string();

    // Replay protection: reject timestamps older than 5 minutes
    let ts: i64 = timestamp.parse().map_err(|_| {
        warn!("Invalid timestamp format");
        StatusCode::UNAUTHORIZED.into_response()
    })?;

    let now = chrono::Utc::now().timestamp();
    if (now - ts).abs() > MAX_TIMESTAMP_AGE_SECONDS {
        warn!("Slack request timestamp too old: {} (now: {})", ts, now);
        return Err(StatusCode::UNAUTHORIZED.into_response());
    }

    // Read body bytes for HMAC computation
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, 1_000_000).await.map_err(|_| {
        warn!("Failed to read request body");
        StatusCode::BAD_REQUEST.into_response()
    })?;

    // Compute HMAC-SHA256
    let sig_basestring = format!("v0:{}:{}", timestamp, String::from_utf8_lossy(&body_bytes));
    let mut mac = HmacSha256::new_from_slice(signing_secret.as_bytes()).map_err(|_| {
        warn!("Invalid HMAC key");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;
    mac.update(sig_basestring.as_bytes());
    let computed = format!("v0={}", hex::encode(mac.finalize().into_bytes()));

    // Constant-time comparison
    if !constant_time_eq(computed.as_bytes(), signature.as_bytes()) {
        warn!("Slack signature verification failed");
        return Err(StatusCode::UNAUTHORIZED.into_response());
    }

    // Re-inject the body so downstream handlers can read it
    let request = Request::from_parts(parts, Body::from(body_bytes));
    Ok(next.run(request).await)
}

/// Constant-time byte comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq_equal() {
        assert!(constant_time_eq(b"hello", b"hello"));
    }

    #[test]
    fn test_constant_time_eq_different() {
        assert!(!constant_time_eq(b"hello", b"world"));
    }

    #[test]
    fn test_constant_time_eq_different_length() {
        assert!(!constant_time_eq(b"hello", b"hi"));
    }
}
