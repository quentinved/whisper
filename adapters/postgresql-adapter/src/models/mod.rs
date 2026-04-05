pub mod managed_secret;
pub mod shared_secret;

#[derive(Debug, thiserror::Error)]
pub enum ModelConversionError {
    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),

    #[error("Invalid nonce: expected 12 bytes, got {0}")]
    InvalidNonce(usize),
}
