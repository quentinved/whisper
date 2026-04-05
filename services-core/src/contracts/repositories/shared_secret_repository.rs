use crate::entities::shared_secret::SharedSecret;
use crate::values_object::shared_secret::secret_id::SecretId;
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
}
