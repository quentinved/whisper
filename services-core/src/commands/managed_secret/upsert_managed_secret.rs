use crate::{
    contracts::repositories::managed_secret_repository::{
        ManagedSecretRepository, ManagedSecretRepositoryError,
    },
    entities::managed_secret::ManagedSecret,
    values_object::shared_secret::secret_id::SecretId,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpsertManagedSecretError {
    #[error("Payload is empty")]
    EmptyPayload,

    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

impl From<ManagedSecretRepositoryError> for UpsertManagedSecretError {
    fn from(err: ManagedSecretRepositoryError) -> Self {
        Self::InternalError {
            reason: err.to_string(),
        }
    }
}

pub struct UpsertManagedSecret {
    id: SecretId,
    payload: Vec<u8>,
    auth_token_hash: String,
}

impl UpsertManagedSecret {
    pub fn new(id: SecretId, payload: Vec<u8>, auth_token_hash: String) -> Self {
        Self {
            id,
            payload,
            auth_token_hash,
        }
    }

    pub async fn handle(
        self,
        repository: &impl ManagedSecretRepository,
    ) -> Result<bool, UpsertManagedSecretError> {
        if self.payload.is_empty() {
            return Err(UpsertManagedSecretError::EmptyPayload);
        }

        let secret = ManagedSecret::new(self.id, self.payload, self.auth_token_hash);
        Ok(repository.upsert(secret).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::managed_secret::test_utils::mocks::MockManagedSecretRepository;

    #[tokio::test]
    async fn test_upsert_success() {
        let repository = MockManagedSecretRepository::new();
        let id = SecretId::generate();
        let payload = vec![1, 2, 3, 4, 5];

        let command = UpsertManagedSecret::new(id, payload.clone(), "token_hash".to_string());
        let result = command.handle(&repository).await;

        assert!(result.is_ok());
        assert!(result.unwrap(), "expected is_created = true for new secret");
        let saved = repository.get_by_id(&id).await.unwrap();
        assert!(saved.is_some());
        assert_eq!(saved.unwrap().payload(), payload);
    }

    #[tokio::test]
    async fn test_upsert_empty_payload_fails() {
        let repository = MockManagedSecretRepository::new();
        let id = SecretId::generate();

        let command = UpsertManagedSecret::new(id, vec![], "token_hash".to_string());
        let result = command.handle(&repository).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UpsertManagedSecretError::EmptyPayload
        ));
    }

    #[tokio::test]
    async fn test_upsert_overwrites_existing() {
        let repository = MockManagedSecretRepository::new();
        let id = SecretId::generate();

        let command1 = UpsertManagedSecret::new(id, vec![1, 2, 3], "token_hash".to_string());
        let result1 = command1.handle(&repository).await.unwrap();
        assert!(result1, "expected is_created = true for first upsert");

        let command2 = UpsertManagedSecret::new(id, vec![4, 5, 6], "token_hash".to_string());
        let result2 = command2.handle(&repository).await.unwrap();
        assert!(!result2, "expected is_created = false for second upsert");

        let saved = repository.get_by_id(&id).await.unwrap();
        assert_eq!(saved.unwrap().payload(), vec![4, 5, 6]);
    }
}
