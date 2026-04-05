use crate::contracts::repositories::shared_secret_repository::SharedSecretRepository;
use thiserror::Error;

pub struct DeleteExpiredSecrets {}

#[derive(Debug, Error)]
pub enum DeleteExpiredSecretsError {
    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

impl Default for DeleteExpiredSecrets {
    fn default() -> Self {
        Self::new()
    }
}

impl DeleteExpiredSecrets {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle(
        &self,
        shared_secret_repository: &impl SharedSecretRepository,
    ) -> Result<u64, DeleteExpiredSecretsError> {
        shared_secret_repository
            .delete_all_expired()
            .await
            .map_err(|err| DeleteExpiredSecretsError::InternalError {
                reason: err.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::shared_secret::test_utils::mocks::MockSharedSecretRepository;
    use crate::entities::shared_secret::SharedSecret;
    use crate::values_object::shared_secret::{
        secret_encrypted::SecretEncrypted, secret_expiration::SecretExpiration, secret_id::SecretId,
    };
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_delete_expired_secrets() {
        let repository = MockSharedSecretRepository::new();

        // Create expired secret (1 hour ago)
        let _expired_time = (Utc::now() - Duration::hours(1)).timestamp();
        let expired_id = SecretId::generate();
        // We need to create this manually since SecretExpiration::new validates against past times
        let expired = SharedSecret::new(
            expired_id,
            SecretEncrypted::new([0u8; 12], vec![4, 5, 6]),
            SecretExpiration::from_datetime(Utc::now() - Duration::hours(1)),
            false,
        );
        repository.insert(expired);

        // Create valid secret (1 hour in future)
        let future_time = (Utc::now() + Duration::hours(1)).timestamp();
        let valid_expiration = SecretExpiration::try_from(future_time).unwrap();
        let valid_id = SecretId::generate();
        let valid = SharedSecret::new(
            valid_id,
            SecretEncrypted::new([1u8; 12], vec![10, 11, 12]),
            valid_expiration,
            false,
        );
        repository.insert(valid);

        assert_eq!(repository.count(), 2);

        // Run delete expired secrets
        let command = DeleteExpiredSecrets::new();
        let result = command.handle(&repository).await;

        assert!(result.is_ok());
        assert_eq!(repository.count(), 1);

        // Verify expired was deleted and valid remains
        assert!(repository.get_by_id(&expired_id).await.unwrap().is_none());
        assert!(repository.get_by_id(&valid_id).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_delete_expired_secrets_no_expired() {
        let repository = MockSharedSecretRepository::new();

        // Create only valid secrets
        let future_time = (Utc::now() + Duration::hours(1)).timestamp();
        let expiration = SecretExpiration::try_from(future_time).unwrap();
        let secret = SharedSecret::new(
            SecretId::generate(),
            SecretEncrypted::new([0u8; 12], vec![4, 5, 6]),
            expiration,
            false,
        );
        repository.insert(secret);

        assert_eq!(repository.count(), 1);

        let command = DeleteExpiredSecrets::new();
        let result = command.handle(&repository).await;

        assert!(result.is_ok());
        assert_eq!(repository.count(), 1); // Nothing deleted
    }
}
