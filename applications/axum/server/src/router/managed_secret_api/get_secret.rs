use super::auth::BearerToken;
use crate::{app_state::AppState, error::CustomError};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use whisper_core::{
    commands::managed_secret::get_managed_secret::GetManagedSecret,
    contracts::repositories::managed_secret_repository::ManagedSecretRepository,
    values_object::shared_secret::secret_id::SecretId,
};

#[derive(Debug, Serialize)]
pub struct GetSecretOutput {
    pub payload: String,
}

pub async fn get_secret(
    State(app_state): State<Arc<AppState>>,
    Path(secret_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, CustomError> {
    let token = BearerToken::extract(&headers)?;
    let id = SecretId::new(secret_id);
    let repository = app_state.managed_secret_repository();

    let stored_hash = repository.get_auth_token_hash(&id).await?;
    token.verify(&stored_hash)?;

    let command = GetManagedSecret::new(id);
    match command.handle(&repository).await? {
        Some(payload) => {
            info!("Managed secret retrieved id={}", secret_id);
            Ok((
                StatusCode::OK,
                Json(GetSecretOutput {
                    payload: base64_url::encode(&payload),
                }),
            )
                .into_response())
        }
        None => Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": { "reason": "Secret not found" } })),
        )
            .into_response()),
    }
}
