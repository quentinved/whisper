use crate::{
    contracts::repositories::managed_secret_repository::{
        ManagedSecretRepository, ManagedSecretRepositoryError,
    },
    values_object::shared_secret::secret_id::SecretId,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetManagedSecretError {
    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

impl From<ManagedSecretRepositoryError> for GetManagedSecretError {
    fn from(err: ManagedSecretRepositoryError) -> Self {
        Self::InternalError {
            reason: err.to_string(),
        }
    }
}

pub struct GetManagedSecret {
    secret_id: SecretId,
}

impl GetManagedSecret {
    pub fn new(secret_id: SecretId) -> Self {
        Self { secret_id }
    }

    pub async fn handle(
        &self,
        repository: &impl ManagedSecretRepository,
    ) -> Result<Option<Vec<u8>>, GetManagedSecretError> {
        let secret = repository.pull_by_id(&self.secret_id).await?;
        Ok(secret.map(|s| s.into_payload()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::managed_secret::test_utils::mocks::MockManagedSecretRepository;
    use crate::entities::managed_secret::ManagedSecret;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_existing_secret() {
        let repository = MockManagedSecretRepository::new();
        let id = SecretId::generate();
        let now = Utc::now();
        let secret = ManagedSecret::from_persisted(id, vec![10, 20, 30], now, now, None, None);
        repository.upsert(secret).await.unwrap();

        let command = GetManagedSecret::new(id);
        let result = command.handle(&repository).await.unwrap();

        assert_eq!(result, Some(vec![10, 20, 30]));
    }

    #[tokio::test]
    async fn test_get_nonexistent_secret() {
        let repository = MockManagedSecretRepository::new();
        let id = SecretId::generate();

        let command = GetManagedSecret::new(id);
        let result = command.handle(&repository).await.unwrap();

        assert_eq!(result, None);
    }
}
