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

/// A retrieved one-time secret.
/// `Plain`: server-encrypted at rest, decrypted here, ready to display.
/// `ClientEncrypted`: zero-knowledge payload (`nonce ‖ ciphertext`); only the
/// link holder's `#k=` fragment can decrypt it — the server has no key.
#[derive(Debug, Clone, PartialEq)]
pub enum RetrievedSecret {
    Plain {
        secret: String,
        self_destruct: bool,
    },
    ClientEncrypted {
        payload: Vec<u8>,
        self_destruct: bool,
    },
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
    ) -> Result<Option<RetrievedSecret>, GetSecretByIdError> {
        let Some(shared_secret) = shared_secret_repository
            .get_by_id(&self.secret_id)
            .await
            .map_err(|err| GetSecretByIdError::InternalError {
                reason: err.to_string(),
            })?
        else {
            return Ok(None);
        };

        // Best-effort: the adapter logs failures, and a reveal must never fail
        // because its receipt could not be stamped (a self-destruct row is
        // already gone at this point).
        let _ = shared_secret_repository
            .mark_receipt_opened(&self.secret_id)
            .await;

        let self_destruct = shared_secret.self_destruct();

        if shared_secret.client_encrypted() {
            let payload = shared_secret.encrypted_secret().into_payload();
            return Ok(Some(RetrievedSecret::ClientEncrypted {
                payload,
                self_destruct,
            }));
        }

        let decrypted_secret = secret_encryption
            .decrypt_secret(shared_secret.encrypted_secret())
            .map_err(|err| GetSecretByIdError::DecryptionFailed {
                reason: err.to_string(),
            })?;

        Ok(Some(RetrievedSecret::Plain {
            secret: decrypted_secret,
            self_destruct,
        }))
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

        assert_eq!(
            result,
            Some(RetrievedSecret::Plain {
                secret: "test_secret".to_string(),
                self_destruct: false
            })
        );
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

        assert_eq!(
            result,
            Some(RetrievedSecret::Plain {
                secret: "secret".to_string(),
                self_destruct: true
            })
        );

        // Note: Self-destruct deletion happens in the repository layer,
        // not in this command, so we can't test it here
    }

    #[tokio::test]
    async fn client_encrypted_secret_is_returned_undecrypted() {
        let encryption = MockEncryption;
        let repository = MockSharedSecretRepository::new();
        let secret_id = SecretId::generate();
        let future_time = (Utc::now() + Duration::hours(1)).timestamp();
        let expiration = SecretExpiration::try_from(future_time).unwrap();

        let nonce = [9u8; 12];
        let cypher = b"opaque-ciphertext".to_vec();
        let secret = SharedSecret::new_client_encrypted(
            secret_id,
            crate::values_object::shared_secret::secret_encrypted::SecretEncrypted::new(
                nonce,
                cypher.clone(),
            ),
            expiration,
            false,
        );
        repository.insert(secret);

        let command = GetSecretById::new(secret_id);
        let result = command.handle(&encryption, &repository).await.unwrap();

        let mut expected_payload = nonce.to_vec();
        expected_payload.extend(cypher);
        assert_eq!(
            result,
            Some(RetrievedSecret::ClientEncrypted {
                payload: expected_payload,
                self_destruct: false
            })
        );
    }

    fn tracked_secret(repository: &MockSharedSecretRepository) -> SecretId {
        let secret_id = SecretId::generate();
        let expiration = SecretExpiration::from_datetime(Utc::now() + Duration::hours(1));
        let encrypted = MockEncryption.encrypt_secret("tracked").unwrap();
        repository.insert(SharedSecret::new(secret_id, encrypted, expiration, false));
        secret_id
    }

    #[tokio::test]
    async fn first_reveal_stamps_the_receipt_once() {
        use crate::commands::shared_secret::create_delivery_receipt::CreateDeliveryReceipt;

        let repository = MockSharedSecretRepository::new();
        let secret_id = tracked_secret(&repository);
        let expiration = SecretExpiration::from_datetime(Utc::now() + Duration::hours(1));
        CreateDeliveryReceipt::new(secret_id, expiration)
            .handle(&repository)
            .await
            .unwrap();
        assert!(repository
            .receipt_for(&secret_id)
            .unwrap()
            .opened_at()
            .is_none());

        let command = GetSecretById::new(secret_id);
        command.handle(&MockEncryption, &repository).await.unwrap();
        let first_open = repository.receipt_for(&secret_id).unwrap().opened_at();
        assert!(first_open.is_some());

        command.handle(&MockEncryption, &repository).await.unwrap();
        assert_eq!(
            repository.receipt_for(&secret_id).unwrap().opened_at(),
            first_open,
            "a second reveal must not move the first-open time"
        );
    }

    #[tokio::test]
    async fn reveal_succeeds_when_the_receipt_cannot_be_stamped() {
        let repository = MockSharedSecretRepository::with_failing_receipts();
        let secret_id = tracked_secret(&repository);

        let result = GetSecretById::new(secret_id)
            .handle(&MockEncryption, &repository)
            .await
            .unwrap();

        assert_eq!(
            result,
            Some(RetrievedSecret::Plain {
                secret: "tracked".to_string(),
                self_destruct: false
            })
        );
    }
}
