use super::auth::BearerToken;
use crate::{app_state::AppState, error::CustomError};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use whisper_core::{
    commands::managed_secret::upsert_managed_secret::UpsertManagedSecret,
    contracts::repositories::managed_secret_repository::ManagedSecretRepository,
    values_object::shared_secret::secret_id::SecretId,
};

#[derive(Debug, Deserialize)]
pub struct UpsertPayload {
    pub payload: String,
}

pub async fn upsert_secret(
    State(app_state): State<Arc<AppState>>,
    Path(secret_id): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<UpsertPayload>,
) -> Result<StatusCode, CustomError> {
    const MAX_PAYLOAD_SIZE: usize = 64 * 1024; // 64 KB

    let token = BearerToken::extract(&headers)?;
    let id = SecretId::new(secret_id);
    let repository = app_state.managed_secret_repository();

    let stored_hash = repository.get_auth_token_hash(&id).await?;
    token.verify(&stored_hash)?;

    let payload = base64_url::decode(&body.payload).map_err(|_| CustomError::ValidationError {
        field_name: "payload".to_string(),
        reason: "Invalid base64 encoding".to_string(),
    })?;

    if payload.len() > MAX_PAYLOAD_SIZE {
        return Err(CustomError::ValidationError {
            field_name: "payload".to_string(),
            reason: format!(
                "Payload too large ({} bytes, max {} bytes)",
                payload.len(),
                MAX_PAYLOAD_SIZE
            ),
        });
    }

    let auth_token_hash: String = token.hash().into();
    let command = UpsertManagedSecret::new(id, payload, auth_token_hash);
    let is_created = command.handle(&repository).await?;

    if is_created {
        info!("Managed secret created id={}", secret_id);
        Ok(StatusCode::CREATED)
    } else {
        info!("Managed secret updated id={}", secret_id);
        Ok(StatusCode::NO_CONTENT)
    }
}
