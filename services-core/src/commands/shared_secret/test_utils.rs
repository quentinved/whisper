#[cfg(test)]
pub mod mocks {
    use crate::contracts::repositories::shared_secret_repository::{
        SharedSecretRepository, SharedSecretRepositoryError,
    };
    use crate::entities::shared_secret::SharedSecret;
    use crate::services::secret_encryption::{SecretEncryption, SecretEncryptionError};
    use crate::values_object::shared_secret::secret_encrypted::SecretEncrypted;
    use crate::values_object::shared_secret::secret_id::SecretId;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock encryption that stores plaintext as ciphertext (no real encryption).
    pub struct MockEncryption;

    impl SecretEncryption for MockEncryption {
        fn encrypt_secret(&self, secret: &str) -> Result<SecretEncrypted, SecretEncryptionError> {
            Ok(SecretEncrypted::new([0u8; 12], secret.as_bytes().to_vec()))
        }

        fn decrypt_secret(
            &self,
            encrypted: SecretEncrypted,
        ) -> Result<String, SecretEncryptionError> {
            String::from_utf8(encrypted.cypher().to_vec()).map_err(|_| {
                SecretEncryptionError::InternalError {
                    reason: "invalid utf8".to_string(),
                }
            })
        }
    }

    /// In-memory mock of SharedSecretRepository with optional deletion tracking.
    pub struct MockSharedSecretRepository {
        secrets: Arc<Mutex<HashMap<SecretId, SharedSecret>>>,
        deleted: Arc<Mutex<Vec<SecretId>>>,
    }

    impl Default for MockSharedSecretRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockSharedSecretRepository {
        pub fn new() -> Self {
            Self {
                secrets: Arc::new(Mutex::new(HashMap::new())),
                deleted: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn insert(&self, secret: SharedSecret) {
            self.secrets.lock().unwrap().insert(secret.id(), secret);
        }

        pub fn count(&self) -> usize {
            self.secrets.lock().unwrap().len()
        }

        pub fn deleted_ids(&self) -> Vec<SecretId> {
            self.deleted.lock().unwrap().clone()
        }
    }

    impl SharedSecretRepository for MockSharedSecretRepository {
        async fn save(
            &self,
            shared_secret: SharedSecret,
        ) -> Result<SecretId, SharedSecretRepositoryError> {
            let id = shared_secret.id();
            self.secrets.lock().unwrap().insert(id, shared_secret);
            Ok(id)
        }

        async fn get_by_id(
            &self,
            id: &SecretId,
        ) -> Result<Option<SharedSecret>, SharedSecretRepositoryError> {
            Ok(self.secrets.lock().unwrap().get(id).cloned())
        }

        async fn delete_by_id(&self, id: &SecretId) -> Result<(), SharedSecretRepositoryError> {
            self.secrets.lock().unwrap().remove(id);
            self.deleted.lock().unwrap().push(*id);
            Ok(())
        }

        async fn get_all(&self) -> Result<Vec<SharedSecret>, SharedSecretRepositoryError> {
            Ok(self.secrets.lock().unwrap().values().cloned().collect())
        }

        async fn get_all_expired(&self) -> Result<Vec<SharedSecret>, SharedSecretRepositoryError> {
            Ok(self
                .secrets
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.expiration().is_expired())
                .cloned()
                .collect())
        }

        async fn delete_all_expired(&self) -> Result<u64, SharedSecretRepositoryError> {
            let mut secrets = self.secrets.lock().unwrap();
            let before = secrets.len();
            secrets.retain(|_, p| !p.expiration().is_expired());
            Ok((before - secrets.len()) as u64)
        }
    }
}
