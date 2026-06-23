use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use whisper_core::{
    commands::shared_secret::get_secret_by_id::{GetSecretById, RetrievedSecret},
    values_object::shared_secret::secret_id::SecretId,
};

use crate::{app_state::AppState, error::CustomError, source::Source};

pub async fn get_secret_by_id(
    State(app_state): State<Arc<AppState>>,
    Path(shared_secret_id): Path<Uuid>,
    Query(query): Query<GetSecretQuery>,
) -> Result<(StatusCode, Json<Output>), CustomError> {
    let source = query.source.unwrap_or_default();
    let shared_secret_id = SecretId::new(shared_secret_id);
    let query = GetSecretById::new(shared_secret_id);
    let shared_secret = query
        .handle(&app_state.aes_gcm(), &app_state.shared_secret_repository())
        .await?;
    let shared_secret = match shared_secret {
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(Output {
                    id: "".to_string(),
                    secret: "".to_string(),
                    self_destruct: false,
                    client_encrypted: false,
                }),
            ))
        }
        Some(shared_secret) => shared_secret,
    };

    app_state.analytics().track(
        "secret_retrieved",
        &source.to_string(),
        serde_json::Value::Null,
    );

    let output = match shared_secret {
        RetrievedSecret::Plain {
            secret,
            self_destruct,
        } => Output {
            id: shared_secret_id.value().to_string(),
            secret,
            self_destruct,
            client_encrypted: false,
        },
        RetrievedSecret::ClientEncrypted {
            payload,
            self_destruct,
        } => Output {
            id: shared_secret_id.value().to_string(),
            secret: base64_url::encode(&payload),
            self_destruct,
            client_encrypted: true,
        },
    };

    Ok((StatusCode::OK, Json(output)))
}

#[derive(Debug, Deserialize)]
pub struct GetSecretQuery {
    pub source: Option<Source>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Output {
    id: String,
    /// Plaintext when `client_encrypted` is false; base64url-no-pad
    /// `nonce[12] ‖ ciphertext` when true (only the link's `#k=` fragment
    /// key can decrypt it — the server has none).
    secret: String,
    self_destruct: bool,
    client_encrypted: bool,
}
