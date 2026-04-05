use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use tracing::error;
use whisper_core::{
    commands::managed_secret::delete_managed_secret::DeleteManagedSecretError,
    commands::managed_secret::get_managed_secret::GetManagedSecretError,
    commands::managed_secret::upsert_managed_secret::UpsertManagedSecretError,
    commands::shared_secret::{
        create_secret::CreateSecretError, get_secret_by_id::GetSecretByIdError,
    },
    contracts::repositories::managed_secret_repository::ManagedSecretRepositoryError,
    values_object::shared_secret::secret_expiration::SecretExpirationError,
};

#[derive(Debug, Error)]
pub enum CustomError {
    // Client error
    #[error("validation error for field '{field_name}': '{reason}'")]
    ValidationError { field_name: String, reason: String },

    #[error("Unauthorized: missing or invalid credentials")]
    Unauthorized,

    #[error("Forbidden: invalid auth token")]
    Forbidden,

    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

impl IntoResponse for CustomError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CustomError::ValidationError { field_name, reason } => (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({
                    "error": {
                        "field_name": field_name,
                        "reason": reason,
                    }
                })),
            )
                .into_response(),
            CustomError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "error": { "reason": "Missing or invalid Authorization header" }
                })),
            )
                .into_response(),
            CustomError::Forbidden => (
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({
                    "error": { "reason": "Invalid auth token for this secret" }
                })),
            )
                .into_response(),
            CustomError::InternalError { reason } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "error": {
                        "reason": reason,
                    }
                })),
            )
                .into_response(),
        }
    }
}

impl From<CreateSecretError> for CustomError {
    fn from(err: CreateSecretError) -> Self {
        match err {
            CreateSecretError::SecretTooLarge { size, max } => {
                error!("Secret too large: {} bytes (max {})", size, max);
                Self::ValidationError {
                    field_name: "secret".to_string(),
                    reason: format!("Secret too large: maximum size is {} KB", max / 1024),
                }
            }
            CreateSecretError::EncryptionFailed { reason } => {
                error!("Encryption failed: {:?}", reason);
                Self::InternalError {
                    reason: "Encryption failed".to_string(),
                }
            }
            CreateSecretError::InternalError { reason } => {
                error!("Internal error: {:?}", reason);
                Self::InternalError {
                    reason: "Internal error".to_string(),
                }
            }
        }
    }
}

impl From<SecretExpirationError> for CustomError {
    fn from(err: SecretExpirationError) -> Self {
        match err {
            SecretExpirationError::InvalidExpiration { reason } => {
                error!("Invalid Expiration: {:?}", reason);
                Self::ValidationError {
                    field_name: "expiration".to_string(),
                    reason,
                }
            }
        }
    }
}

impl From<GetSecretByIdError> for CustomError {
    fn from(err: GetSecretByIdError) -> Self {
        error!("Get secret by id error: {:?}", err);
        match err {
            GetSecretByIdError::DecryptionFailed { reason } => {
                error!("DecryptionFailed failed: {:?}", reason);
                Self::InternalError {
                    reason: "Decryption failed".to_string(),
                }
            }
            GetSecretByIdError::InternalError { reason } => {
                error!("Internal error: {:?}", reason);
                Self::InternalError {
                    reason: "Internal error".to_string(),
                }
            }
        }
    }
}

impl From<UpsertManagedSecretError> for CustomError {
    fn from(err: UpsertManagedSecretError) -> Self {
        match err {
            UpsertManagedSecretError::EmptyPayload => Self::ValidationError {
                field_name: "payload".to_string(),
                reason: "Payload cannot be empty".to_string(),
            },
            UpsertManagedSecretError::InternalError { reason } => {
                error!("Upsert managed secret error: {:?}", reason);
                Self::InternalError {
                    reason: "Internal error".to_string(),
                }
            }
        }
    }
}

impl From<GetManagedSecretError> for CustomError {
    fn from(err: GetManagedSecretError) -> Self {
        error!("Get managed secret error: {:?}", err);
        Self::InternalError {
            reason: "Internal error".to_string(),
        }
    }
}

impl From<DeleteManagedSecretError> for CustomError {
    fn from(err: DeleteManagedSecretError) -> Self {
        error!("Delete managed secret error: {:?}", err);
        Self::InternalError {
            reason: "Internal error".to_string(),
        }
    }
}

impl From<ManagedSecretRepositoryError> for CustomError {
    fn from(err: ManagedSecretRepositoryError) -> Self {
        error!("Managed secret repository error: {:?}", err);
        Self::InternalError {
            reason: "Internal error".to_string(),
        }
    }
}
