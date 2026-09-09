use crate::contracts::repositories::shared_secret_repository::{
    SecretMetadata, SharedSecretRepository,
};
use crate::values_object::shared_secret::secret_id::SecretId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetSecretMetadataError {
    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

pub struct GetSecretMetadata {
    secret_id: SecretId,
}

impl GetSecretMetadata {
    pub fn new(secret_id: SecretId) -> Self {
        Self { secret_id }
    }

    pub async fn handle(
        &self,
        repo: &impl SharedSecretRepository,
    ) -> Result<Option<SecretMetadata>, GetSecretMetadataError> {
        repo.get_metadata_by_id(&self.secret_id).await.map_err(|e| {
            GetSecretMetadataError::InternalError {
                reason: e.to_string(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::shared_secret::test_utils::mocks::MockSharedSecretRepository;
    use crate::entities::shared_secret::SharedSecret;
    use crate::values_object::shared_secret::{
        secret_encrypted::SecretEncrypted, secret_expiration::SecretExpiration,
    };
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn returns_flags_without_consuming_self_destruct_secret() {
        let repo = MockSharedSecretRepository::new();
        let id = SecretId::generate();
        let exp =
            SecretExpiration::try_from((Utc::now() + Duration::hours(1)).timestamp()).unwrap();
        repo.insert(SharedSecret::new_client_encrypted(
            id,
            SecretEncrypted::new([0u8; 12], vec![1, 2, 3]),
            exp,
            true,
        ));

        let meta = GetSecretMetadata::new(id)
            .handle(&repo)
            .await
            .unwrap()
            .unwrap();
        assert!(meta.client_encrypted);
        assert!(meta.self_destruct);
        // Not consumed: still present.
        assert!(repo.get_metadata_by_id(&id).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn returns_none_for_missing() {
        let repo = MockSharedSecretRepository::new();
        assert!(GetSecretMetadata::new(SecretId::generate())
            .handle(&repo)
            .await
            .unwrap()
            .is_none());
    }
}
