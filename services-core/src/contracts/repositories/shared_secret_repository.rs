use crate::entities::delivery_receipt::DeliveryReceipt;
use crate::entities::shared_secret::SharedSecret;
use crate::values_object::shared_secret::secret_id::SecretId;
use crate::values_object::shared_secret::status_token::StatusToken;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SharedSecretRepositoryError {
    #[error("Service error: '`{0}`'")]
    ServiceError(String),

    #[error("Secret not found")]
    NotFound,

    #[error("Database error: '`{0}`'")]
    DatabaseError(String),
}

pub type Result<T> = std::result::Result<T, SharedSecretRepositoryError>;

/// Public metadata about a stored secret — safe to expose without consuming it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SecretMetadata {
    pub client_encrypted: bool,
    pub self_destruct: bool,
}

pub trait SharedSecretRepository {
    /// Save a secret
    /// # Arguments
    /// * `secret` - The secret to be saved
    ///
    /// # Return
    /// The created secret's id
    fn save(
        &self,
        secret: SharedSecret,
    ) -> impl std::future::Future<Output = Result<SecretId>> + Send;

    /// Get a secret by its id
    /// # Arguments
    /// * `id` - The secret's id
    ///
    /// # Return
    /// The secret, if it exists
    fn get_by_id(
        &self,
        id: &SecretId,
    ) -> impl std::future::Future<Output = Result<Option<SharedSecret>>> + Send;

    /// Non-consuming metadata lookup: never deletes a self-destruct secret and
    /// never returns ciphertext. Returns None if absent/expired.
    fn get_metadata_by_id(
        &self,
        id: &SecretId,
    ) -> impl std::future::Future<Output = Result<Option<SecretMetadata>>> + Send;

    /// Delete a secret by its id
    /// # Arguments
    /// * `id` - The secret's id
    ///
    /// # Return
    /// () if sucess
    fn delete_by_id(&self, id: &SecretId) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get all secrets
    ///
    /// # Return
    /// A list of all secrets
    fn get_all(&self) -> impl std::future::Future<Output = Result<Vec<SharedSecret>>> + Send;

    // Get all expired secrets
    ///
    /// # Return
    /// A list of all expired secrets
    fn get_all_expired(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<SharedSecret>>> + Send;

    /// Delete all expired secrets in a single operation
    ///
    /// # Return
    /// The number of deleted secrets
    fn delete_all_expired(&self) -> impl std::future::Future<Output = Result<u64>> + Send;

    /// Store the delivery receipt of a tracked secret
    fn save_receipt(
        &self,
        receipt: DeliveryReceipt,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Stamp the first reveal of a secret. No-op when the secret is untracked
    /// or was already opened.
    fn mark_receipt_opened(
        &self,
        secret_id: &SecretId,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get a delivery receipt by its status token
    fn get_receipt(
        &self,
        status_token: &StatusToken,
    ) -> impl std::future::Future<Output = Result<Option<DeliveryReceipt>>> + Send;

    /// Delete receipts whose secret expired before `cutoff`
    ///
    /// # Return
    /// The number of deleted receipts
    fn delete_receipts_expired_before(
        &self,
        cutoff: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<u64>> + Send;
}
