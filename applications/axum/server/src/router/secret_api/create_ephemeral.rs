use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use whisper_core::{
    commands::shared_secret::create_client_encrypted_secret::CreateClientEncryptedSecret,
    values_object::shared_secret::secret_expiration::SecretExpiration,
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
}

#[derive(Debug, Serialize)]
pub struct CreateEphemeralResponse {
    pub id: String,
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
    const MAX_PAYLOAD_B64_LEN: usize = 64 * 1024 * 4 / 3 + 4;
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

    Ok((
        StatusCode::CREATED,
        Json(CreateEphemeralResponse {
            id: secret_id.value().to_string(),
        }),
    ))
}
