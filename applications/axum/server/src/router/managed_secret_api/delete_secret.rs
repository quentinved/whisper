use super::auth::BearerToken;
use crate::{app_state::AppState, error::CustomError};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use whisper_core::{
    commands::managed_secret::delete_managed_secret::DeleteManagedSecret,
    contracts::repositories::managed_secret_repository::ManagedSecretRepository,
    values_object::shared_secret::secret_id::SecretId,
};

pub async fn delete_secret(
    State(app_state): State<Arc<AppState>>,
    Path(secret_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, CustomError> {
    let token = BearerToken::extract(&headers)?;
    let id = SecretId::new(secret_id);
    let repository = app_state.managed_secret_repository();

    let stored_hash = repository.get_auth_token_hash(&id).await?;
    token.verify(&stored_hash)?;

    let command = DeleteManagedSecret::new(id);
    command.handle(&repository).await?;

    info!("Managed secret deleted id={}", secret_id);
    Ok(StatusCode::NO_CONTENT)
}
