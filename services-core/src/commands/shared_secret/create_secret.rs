use crate::{
    contracts::repositories::shared_secret_repository::{
        SharedSecretRepository, SharedSecretRepositoryError,
    },
    entities::shared_secret::SharedSecret,
    services::secret_encryption::SecretEncryption,
    values_object::shared_secret::{secret_expiration::SecretExpiration, secret_id::SecretId},
};
use thiserror::Error;

const MAX_SECRET_SIZE: usize = 64 * 1024; // 64 KB

#[derive(Debug, Error)]
pub enum CreateSecretError {
    #[error("Secret too large: {size} bytes exceeds maximum of {max} bytes")]
    SecretTooLarge { size: usize, max: usize },

    #[error("Encryption Failed: {reason}")]
    EncryptionFailed { reason: String },

    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

pub struct CreateSecret {
    secret: String,
    expiration: SecretExpiration,
    self_destruct: bool,
}

impl From<SharedSecretRepositoryError> for CreateSecretError {
    fn from(err: SharedSecretRepositoryError) -> Self {
        match err {
            SharedSecretRepositoryError::ServiceError(err) => Self::InternalError { reason: err },
            _ => Self::InternalError {
                reason: err.to_string(),
            },
        }
    }
}

impl CreateSecret {
    pub fn new(
        secret: String,
        expiration: SecretExpiration,
        self_destruct: bool,
    ) -> Result<Self, CreateSecretError> {
        if secret.len() > MAX_SECRET_SIZE {
            return Err(CreateSecretError::SecretTooLarge {
                size: secret.len(),
                max: MAX_SECRET_SIZE,
            });
        }
        Ok(Self {
            secret,
            expiration,
            self_destruct,
        })
    }
    pub async fn handle(
        self,
        secret_encryption: &impl SecretEncryption,
        shared_secret_repository: &impl SharedSecretRepository,
    ) -> Result<SecretId, CreateSecretError> {
        let encrypted_secret = secret_encryption
            .encrypt_secret(&self.secret)
            .map_err(|err| CreateSecretError::EncryptionFailed {
                reason: err.to_string(),
            })?;
        let shared_secret_id = SecretId::generate();
        let shared_secret = SharedSecret::new(
            shared_secret_id,
            encrypted_secret,
            self.expiration,
            self.self_destruct,
        );
        Ok(shared_secret_repository.save(shared_secret).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::shared_secret::test_utils::mocks::{
        MockEncryption, MockSharedSecretRepository,
    };
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_create_secret_success() {
        let encryption = MockEncryption;
        let repository = MockSharedSecretRepository::new();
        let future_time = (Utc::now() + Duration::hours(1)).timestamp();
        let expiration = SecretExpiration::try_from(future_time).unwrap();

        let command = CreateSecret::new("test_secret".to_string(), expiration, false).unwrap();
        let result = command.handle(&encryption, &repository).await;

        assert!(result.is_ok());
        let secret_id = result.unwrap();

        // Verify it was saved
        let saved = repository.get_by_id(&secret_id).await.unwrap();
        assert!(saved.is_some());
        assert!(!saved.unwrap().self_destruct());
    }

    #[tokio::test]
    async fn test_create_secret_with_self_destruct() {
        let encryption = MockEncryption;
        let repository = MockSharedSecretRepository::new();
        let future_time = (Utc::now() + Duration::hours(1)).timestamp();
        let expiration = SecretExpiration::try_from(future_time).unwrap();

        let command = CreateSecret::new("my_secret".to_string(), expiration, true).unwrap();
        let result = command.handle(&encryption, &repository).await;

        assert!(result.is_ok());
        let secret_id = result.unwrap();

        // Verify self_destruct flag is set
        let saved = repository.get_by_id(&secret_id).await.unwrap();
        assert!(saved.is_some());
        assert!(saved.unwrap().self_destruct());
    }
}
