use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use whisper_core::{
    commands::shared_secret::{
        create_client_encrypted_secret::CreateClientEncryptedSecret,
        create_delivery_receipt::CreateDeliveryReceipt, create_secret::MAX_SECRET_SIZE,
    },
    values_object::shared_secret::{secret_expiration::SecretExpiration, secret_id::SecretId},
};

use crate::{app_state::AppState, error::CustomError, source::Source};

#[derive(Debug, Deserialize)]
pub struct CreateEphemeralQuery {
    pub source: Option<Source>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEphemeralBody {
    /// base64url-no-pad encoded `nonce[12] ‖ ciphertext`, encrypted on the sender's device.
    pub payload: String,
    /// Unix timestamp (seconds).
    pub expiration: i64,
    #[serde(default)]
    pub self_destruct: bool,
    /// Opt-in: record when the secret is first opened and return a sender-only
    /// `status_token` to read it. Clients that don't send it see no change.
    #[serde(default)]
    pub track_delivery: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateEphemeralResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_token: Option<String>,
}

pub async fn create_ephemeral(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<CreateEphemeralQuery>,
    Json(body): Json<CreateEphemeralBody>,
) -> Result<(StatusCode, Json<CreateEphemeralResponse>), CustomError> {
    let source = query.source.unwrap_or_default();

    // Reject grossly oversized payloads before decoding. The command enforces
    // the exact 64 KB cap on the decoded bytes; this pre-check just avoids the
    // decode allocation for bodies that can't possibly fit (base64 expands by
    // 4/3). Axum's default body limit bounds the request size above that.
    const MAX_PAYLOAD_B64_LEN: usize = MAX_SECRET_SIZE * 4 / 3 + 4;
    if body.payload.len() > MAX_PAYLOAD_B64_LEN {
        return Err(CustomError::ValidationError {
            field_name: "payload".to_string(),
            reason: "Payload too large: maximum size is 64 KB".to_string(),
        });
    }

    let payload = base64_url::decode(&body.payload).map_err(|_| CustomError::ValidationError {
        field_name: "payload".to_string(),
        reason: "payload must be base64url-encoded".to_string(),
    })?;
    let expiration = SecretExpiration::try_from(body.expiration)?;

    let command = CreateClientEncryptedSecret::new(payload, expiration, body.self_destruct)?;
    let secret_id = command
        .handle(&app_state.shared_secret_repository())
        .await?;

    info!(
        "Client-encrypted secret created id={} source={}",
        secret_id.value(),
        source
    );
    app_state.analytics().track(
        "secret_created",
        &source.to_string(),
        serde_json::json!({
            "self_destruct": body.self_destruct,
            "expiration": body.expiration,
            "client_encrypted": true,
        }),
    );

    let status_token = match body.track_delivery {
        true => create_receipt(&app_state, secret_id, expiration).await,
        false => None,
    };

    Ok((
        StatusCode::CREATED,
        Json(CreateEphemeralResponse {
            id: secret_id.value().to_string(),
            status_token,
        }),
    ))
}

/// The secret is already stored: if its receipt can't be, the sender still
/// gets their link, just without delivery tracking.
async fn create_receipt(
    app_state: &AppState,
    secret_id: SecretId,
    expiration: SecretExpiration,
) -> Option<String> {
    CreateDeliveryReceipt::new(secret_id, expiration)
        .handle(&app_state.shared_secret_repository())
        .await
        .map(|token| token.to_string())
        .map_err(|err| error!("Delivery receipt not created id={}: {}", secret_id, err))
        .ok()
}
