#[cfg(any(test, feature = "test-utils"))]
pub mod mocks {
    use crate::contracts::repositories::shared_secret_repository::{
        SecretMetadata, SharedSecretRepository, SharedSecretRepositoryError,
    };
    use crate::entities::delivery_receipt::DeliveryReceipt;
    use crate::entities::shared_secret::SharedSecret;
    use crate::services::secret_encryption::{SecretEncryption, SecretEncryptionError};
    use crate::values_object::shared_secret::secret_encrypted::SecretEncrypted;
    use crate::values_object::shared_secret::secret_id::SecretId;
    use crate::values_object::shared_secret::status_token::StatusToken;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock encryption that stores plaintext as ciphertext (no real encryption).
    #[derive(Debug)]
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
    #[derive(Debug)]
    pub struct MockSharedSecretRepository {
        secrets: Arc<Mutex<HashMap<SecretId, SharedSecret>>>,
        deleted: Arc<Mutex<Vec<SecretId>>>,
        receipts: Arc<Mutex<HashMap<StatusToken, DeliveryReceipt>>>,
        fail_receipts: bool,
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
                receipts: Arc::new(Mutex::new(HashMap::new())),
                fail_receipts: false,
            }
        }

        /// Every receipt operation fails, to prove reveals don't depend on it.
        pub fn with_failing_receipts() -> Self {
            Self {
                fail_receipts: true,
                ..Self::new()
            }
        }

        pub fn receipt_for(&self, secret_id: &SecretId) -> Option<DeliveryReceipt> {
            self.receipts
                .lock()
                .unwrap()
                .values()
                .find(|r| r.secret_id() == *secret_id)
                .cloned()
        }

        fn check_receipts(&self) -> Result<(), SharedSecretRepositoryError> {
            match self.fail_receipts {
                true => Err(SharedSecretRepositoryError::DatabaseError(
                    "receipts unavailable".to_string(),
                )),
                false => Ok(()),
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

        async fn get_metadata_by_id(
            &self,
            id: &SecretId,
        ) -> Result<Option<SecretMetadata>, SharedSecretRepositoryError> {
            Ok(self
                .secrets
                .lock()
                .unwrap()
                .get(id)
                .map(|s| SecretMetadata {
                    client_encrypted: s.client_encrypted(),
                    self_destruct: s.self_destruct(),
                }))
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

        async fn save_receipt(
            &self,
            receipt: DeliveryReceipt,
        ) -> Result<(), SharedSecretRepositoryError> {
            self.check_receipts()?;
            self.receipts
                .lock()
                .unwrap()
                .insert(receipt.status_token(), receipt);
            Ok(())
        }

        async fn mark_receipt_opened(
            &self,
            secret_id: &SecretId,
        ) -> Result<(), SharedSecretRepositoryError> {
            self.check_receipts()?;
            let mut receipts = self.receipts.lock().unwrap();
            if let Some(r) = receipts
                .values_mut()
                .find(|r| r.secret_id() == *secret_id && r.opened_at().is_none())
            {
                *r = DeliveryReceipt::new(
                    r.status_token(),
                    r.secret_id(),
                    r.expiration(),
                    Some(Utc::now()),
                );
            }
            Ok(())
        }

        async fn get_receipt(
            &self,
            status_token: &StatusToken,
        ) -> Result<Option<DeliveryReceipt>, SharedSecretRepositoryError> {
            self.check_receipts()?;
            Ok(self.receipts.lock().unwrap().get(status_token).cloned())
        }

        async fn delete_receipts_expired_before(
            &self,
            cutoff: DateTime<Utc>,
        ) -> Result<u64, SharedSecretRepositoryError> {
            self.check_receipts()?;
            let mut receipts = self.receipts.lock().unwrap();
            let before = receipts.len();
            receipts.retain(|_, r| r.expiration().value() >= cutoff);
            Ok((before - receipts.len()) as u64)
        }
    }
}
