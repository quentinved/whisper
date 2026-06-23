use crate::{
    contracts::repositories::shared_secret_repository::{
        SharedSecretRepository, SharedSecretRepositoryError,
    },
    entities::shared_secret::SharedSecret,
    values_object::shared_secret::{
        secret_encrypted::{SecretEncrypted, NONCE_SIZE},
        secret_expiration::SecretExpiration,
        secret_id::SecretId,
    },
};
use thiserror::Error;

const MAX_PAYLOAD_SIZE: usize = 64 * 1024; // 64 KB, same limit as CreateSecret
/// nonce (12) + at least 1 byte of ciphertext
const MIN_PAYLOAD_SIZE: usize = NONCE_SIZE + 1;

#[derive(Debug, Error)]
pub enum CreateClientEncryptedSecretError {
    #[error("Payload too large: {size} bytes exceeds maximum of {max} bytes")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("Payload too short: must be at least 13 bytes (12-byte nonce + ciphertext)")]
    PayloadTooShort,

    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

impl From<SharedSecretRepositoryError> for CreateClientEncryptedSecretError {
    fn from(err: SharedSecretRepositoryError) -> Self {
        Self::InternalError {
            reason: err.to_string(),
        }
    }
}

/// Stores a payload that was encrypted on the sender's device (zero-knowledge).
/// The payload layout is `nonce[12] ‖ ciphertext`; the server never sees the key.
pub struct CreateClientEncryptedSecret {
    payload: Vec<u8>,
    expiration: SecretExpiration,
    self_destruct: bool,
}

impl CreateClientEncryptedSecret {
    pub fn new(
        payload: Vec<u8>,
        expiration: SecretExpiration,
        self_destruct: bool,
    ) -> Result<Self, CreateClientEncryptedSecretError> {
        if payload.len() < MIN_PAYLOAD_SIZE {
            return Err(CreateClientEncryptedSecretError::PayloadTooShort);
        }
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(CreateClientEncryptedSecretError::PayloadTooLarge {
                size: payload.len(),
                max: MAX_PAYLOAD_SIZE,
            });
        }
        Ok(Self {
            payload,
            expiration,
            self_destruct,
        })
    }

    pub async fn handle(
        self,
        shared_secret_repository: &impl SharedSecretRepository,
    ) -> Result<SecretId, CreateClientEncryptedSecretError> {
        let nonce: [u8; NONCE_SIZE] = self.payload[..NONCE_SIZE].try_into().map_err(|_| {
            CreateClientEncryptedSecretError::InternalError {
                reason: "nonce length mismatch".to_string(),
            }
        })?;
        let cypher = self.payload[NONCE_SIZE..].to_vec();
        let encrypted = SecretEncrypted::new(nonce, cypher);

        let id = SecretId::generate();
        let shared_secret =
            SharedSecret::new_client_encrypted(id, encrypted, self.expiration, self.self_destruct);
        Ok(shared_secret_repository.save(shared_secret).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::shared_secret::test_utils::mocks::MockSharedSecretRepository;
    use chrono::{Duration, Utc};

    fn future_expiration() -> SecretExpiration {
        let ts = (Utc::now() + Duration::hours(1)).timestamp();
        SecretExpiration::try_from(ts).unwrap()
    }

    #[tokio::test]
    async fn stores_payload_with_client_encrypted_flag() {
        let repository = MockSharedSecretRepository::new();
        let mut payload = vec![7u8; NONCE_SIZE]; // nonce = twelve 0x07 bytes
        payload.extend_from_slice(b"ciphertext-bytes");

        let command = CreateClientEncryptedSecret::new(payload, future_expiration(), true).unwrap();
        let id = command.handle(&repository).await.unwrap();

        let saved = repository.get_by_id(&id).await.unwrap().unwrap();
        assert!(saved.client_encrypted());
        assert!(saved.self_destruct());
        let (nonce, cypher) = saved.encrypted_secret().into_parts();
        assert_eq!(nonce, [7u8; NONCE_SIZE]);
        assert_eq!(cypher, b"ciphertext-bytes".to_vec());
    }

    #[tokio::test]
    async fn rejects_payload_shorter_than_nonce_plus_one() {
        let result =
            CreateClientEncryptedSecret::new(vec![0u8; NONCE_SIZE], future_expiration(), false);
        assert!(matches!(
            result,
            Err(CreateClientEncryptedSecretError::PayloadTooShort)
        ));
    }

    #[tokio::test]
    async fn rejects_payload_over_64kb() {
        let result = CreateClientEncryptedSecret::new(
            vec![0u8; MAX_PAYLOAD_SIZE + 1],
            future_expiration(),
            false,
        );
        assert!(matches!(
            result,
            Err(CreateClientEncryptedSecretError::PayloadTooLarge { .. })
        ));
    }

    #[tokio::test]
    async fn accepts_minimum_payload_of_nonce_plus_one_byte() {
        let result = CreateClientEncryptedSecret::new(
            vec![0u8; MIN_PAYLOAD_SIZE],
            future_expiration(),
            false,
        );
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn accepts_payload_of_exactly_64kb() {
        let result = CreateClientEncryptedSecret::new(
            vec![0u8; MAX_PAYLOAD_SIZE],
            future_expiration(),
            false,
        );
        assert!(result.is_ok());
    }
}
