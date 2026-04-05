use crate::{
    contracts::repositories::managed_secret_repository::{
        ManagedSecretRepository, ManagedSecretRepositoryError,
    },
    values_object::shared_secret::secret_id::SecretId,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteManagedSecretError {
    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

impl From<ManagedSecretRepositoryError> for DeleteManagedSecretError {
    fn from(err: ManagedSecretRepositoryError) -> Self {
        Self::InternalError {
            reason: err.to_string(),
        }
    }
}

pub struct DeleteManagedSecret {
    secret_id: SecretId,
}

impl DeleteManagedSecret {
    pub fn new(secret_id: SecretId) -> Self {
        Self { secret_id }
    }

    pub async fn handle(
        &self,
        repository: &impl ManagedSecretRepository,
    ) -> Result<(), DeleteManagedSecretError> {
        repository.delete_by_id(&self.secret_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::managed_secret::test_utils::mocks::MockManagedSecretRepository;
    use crate::entities::managed_secret::ManagedSecret;
    use chrono::Utc;

    #[tokio::test]
    async fn test_delete_existing_secret() {
        let repository = MockManagedSecretRepository::new();
        let id = SecretId::generate();
        let now = Utc::now();
        let secret = ManagedSecret::from_persisted(id, vec![1, 2, 3], now, now, None, None);
        repository.upsert(secret).await.unwrap();

        let command = DeleteManagedSecret::new(id);
        let result = command.handle(&repository).await;

        assert!(result.is_ok());
        let found = repository.get_by_id(&id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_secret_is_idempotent() {
        let repository = MockManagedSecretRepository::new();
        let id = SecretId::generate();

        let command = DeleteManagedSecret::new(id);
        let result = command.handle(&repository).await;

        assert!(result.is_ok());
    }
}
