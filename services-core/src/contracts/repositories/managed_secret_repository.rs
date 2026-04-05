use crate::entities::managed_secret::ManagedSecret;
use crate::values_object::shared_secret::secret_id::SecretId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManagedSecretRepositoryError {
    #[error("Service error: '{0}'")]
    ServiceError(String),

    #[error("Database error: '{0}'")]
    DatabaseError(String),
}

pub type Result<T> = std::result::Result<T, ManagedSecretRepositoryError>;

pub trait ManagedSecretRepository {
    fn upsert(
        &self,
        secret: ManagedSecret,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn get_by_id(
        &self,
        id: &SecretId,
    ) -> impl std::future::Future<Output = Result<Option<ManagedSecret>>> + Send;

    /// Get a secret by its id and atomically update last_pulled_at
    fn pull_by_id(
        &self,
        id: &SecretId,
    ) -> impl std::future::Future<Output = Result<Option<ManagedSecret>>> + Send;

    fn delete_by_id(&self, id: &SecretId) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Returns `None` if no row exists, `Some(None)` if the row has a NULL hash,
    /// or `Some(Some(hash))` if a hash is stored.
    fn get_auth_token_hash(
        &self,
        id: &SecretId,
    ) -> impl std::future::Future<Output = Result<Option<Option<String>>>> + Send;
}
