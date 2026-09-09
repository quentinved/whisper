use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use whisper_core::{
    commands::shared_secret::get_secret_metadata::GetSecretMetadata,
    values_object::shared_secret::secret_id::SecretId,
};

use crate::{app_state::AppState, error::CustomError};

#[derive(Debug, Serialize)]
pub struct Output {
    exists: bool,
    client_encrypted: bool,
    self_destruct: bool,
}

/// Non-consuming: reports flags so the client can refuse to burn a
/// zero-knowledge self-destruct secret when the `#k=` key is missing.
pub async fn get_secret_metadata(
    State(app_state): State<Arc<AppState>>,
    Path(shared_secret_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Output>), CustomError> {
    let id = SecretId::new(shared_secret_id);
    let meta = GetSecretMetadata::new(id)
        .handle(&app_state.shared_secret_repository())
        .await
        .map_err(|e| CustomError::InternalError {
            reason: e.to_string(),
        })?;
    Ok(match meta {
        Some(m) => (
            StatusCode::OK,
            Json(Output {
                exists: true,
                client_encrypted: m.client_encrypted,
                self_destruct: m.self_destruct,
            }),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(Output {
                exists: false,
                client_encrypted: false,
                self_destruct: false,
            }),
        ),
    })
}
