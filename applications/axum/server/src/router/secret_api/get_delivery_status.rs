use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use whisper_core::{
    commands::shared_secret::get_delivery_receipt::GetDeliveryReceipt,
    entities::delivery_receipt::{DeliveryReceipt, DeliveryStatus},
    values_object::shared_secret::status_token::StatusToken,
};

use crate::{app_state::AppState, error::CustomError};

#[derive(Debug, Serialize)]
pub struct Output {
    state: &'static str,
    /// RFC 3339
    expires_at: String,
    /// RFC 3339, set once the secret has been revealed
    opened_at: Option<String>,
}

impl From<DeliveryReceipt> for Output {
    fn from(receipt: DeliveryReceipt) -> Self {
        // Whole seconds + `Z`: what every client's ISO 8601 parser accepts
        // (Swift's `.iso8601` rejects fractional seconds).
        let rfc3339 = |at: DateTime<Utc>| at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let (state, opened_at) = match receipt.status() {
            DeliveryStatus::Waiting => ("waiting", None),
            DeliveryStatus::Opened { opened_at } => ("opened", Some(rfc3339(opened_at))),
            DeliveryStatus::Expired => ("expired", None),
        };
        Self {
            state,
            expires_at: rfc3339(receipt.expiration().value()),
            opened_at,
        }
    }
}

/// Sender-only: the status token reads the delivery receipt, never the secret.
/// 404 once the receipt is purged (7 days after expiry) or for unknown tokens.
pub async fn get_delivery_status(
    State(app_state): State<Arc<AppState>>,
    Path(status_token): Path<Uuid>,
) -> Result<Response, CustomError> {
    let receipt = GetDeliveryReceipt::new(StatusToken::new(status_token))
        .handle(&app_state.shared_secret_repository())
        .await
        .map_err(|e| CustomError::InternalError {
            reason: e.to_string(),
        })?;
    let no_store = [(header::CACHE_CONTROL, "no-store")];
    Ok(match receipt {
        Some(receipt) => (no_store, Json(Output::from(receipt))).into_response(),
        None => (StatusCode::NOT_FOUND, no_store).into_response(),
    })
}
