use thiserror::Error;

use crate::values_object::shared_secret::secret_encrypted::SecretEncrypted;

#[derive(Debug, Error)]
pub enum SecretEncryptionError {
    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

pub type Result<T> = std::result::Result<T, SecretEncryptionError>;

pub trait SecretEncryption {
    fn encrypt_secret(&self, secret: &str) -> Result<SecretEncrypted>;
    fn decrypt_secret(&self, encrypted_secret: SecretEncrypted) -> Result<String>;
}
