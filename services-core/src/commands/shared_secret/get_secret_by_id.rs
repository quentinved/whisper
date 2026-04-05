use crate::{
    contracts::repositories::shared_secret_repository::SharedSecretRepository,
    services::secret_encryption::SecretEncryption,
    values_object::shared_secret::secret_id::SecretId,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetSecretByIdError {
    #[error("Decryption Failed: {reason}")]
    DecryptionFailed { reason: String },

    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

pub struct GetSecretById {
    secret_id: SecretId,
}

impl GetSecretById {
    pub fn new(secret_id: SecretId) -> Self {
        Self { secret_id }
    }

    pub async fn handle(
        &self,
        secret_encryption: &impl SecretEncryption,
        shared_secret_repository: &impl SharedSecretRepository,
    ) -> Result<Option<(String, bool)>, GetSecretByIdError> {
        let Some(shared_secret) = shared_secret_repository
            .get_by_id(&self.secret_id)
            .await
            .map_err(|err| GetSecretByIdError::InternalError {
                reason: err.to_string(),
            })?
        else {
            return Ok(None);
        };

        let self_destruct = shared_secret.self_destruct();
        let decrypted_secret = secret_encryption
            .decrypt_secret(shared_secret.encrypted_secret())
            .map_err(|err| GetSecretByIdError::DecryptionFailed {
                reason: err.to_string(),
            })?;

        Ok(Some((decrypted_secret, self_destruct)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::shared_secret::test_utils::mocks::{
        MockEncryption, MockSharedSecretRepository,
    };
    use crate::entities::shared_secret::SharedSecret;
    use crate::services::secret_encryption::SecretEncryption;
    use crate::values_object::shared_secret::secret_expiration::SecretExpiration;
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_get_secret_success() {
        let encryption = MockEncryption;
        let repository = MockSharedSecretRepository::new();
        let secret_id = SecretId::generate();
        let future_time = (Utc::now() + Duration::hours(1)).timestamp();
        let expiration = SecretExpiration::try_from(future_time).unwrap();
        let encrypted = encryption.encrypt_secret("test_secret").unwrap();

        let secret = SharedSecret::new(secret_id, encrypted, expiration, false);
        repository.insert(secret);

        let command = GetSecretById::new(secret_id);
        let result = command.handle(&encryption, &repository).await.unwrap();

        assert!(result.is_some());
        let (decrypted, self_destruct) = result.unwrap();
        assert_eq!(decrypted, "test_secret");
        assert!(!self_destruct);
    }

    #[tokio::test]
    async fn test_get_secret_not_found() {
        let encryption = MockEncryption;
        let repository = MockSharedSecretRepository::new();
        let secret_id = SecretId::generate();

        let command = GetSecretById::new(secret_id);
        let result = command.handle(&encryption, &repository).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_secret_with_self_destruct() {
        let encryption = MockEncryption;
        let repository = MockSharedSecretRepository::new();
        let secret_id = SecretId::generate();
        let future_time = (Utc::now() + Duration::hours(1)).timestamp();
        let expiration = SecretExpiration::try_from(future_time).unwrap();
        let encrypted = encryption.encrypt_secret("secret").unwrap();

        let secret = SharedSecret::new(secret_id, encrypted, expiration, true);
        repository.insert(secret);

        let command = GetSecretById::new(secret_id);
        let result = command.handle(&encryption, &repository).await.unwrap();

        assert!(result.is_some());
        let (decrypted, self_destruct) = result.unwrap();
        assert_eq!(decrypted, "secret");
        assert!(self_destruct);

        // Note: Self-destruct deletion happens in the repository layer,
        // not in this command, so we can't test it here
    }
}
