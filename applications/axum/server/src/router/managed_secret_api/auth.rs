use std::fmt;

use axum::http::{header, HeaderMap};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::error::CustomError;

/// A SHA-256 hash of a bearer token, used for storage and comparison.
pub struct AuthTokenHash(String);

impl fmt::Display for AuthTokenHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<AuthTokenHash> for String {
    fn from(hash: AuthTokenHash) -> Self {
        hash.0
    }
}

pub struct BearerToken(String);

impl BearerToken {
    /// Extract the bearer token from the Authorization header.
    pub fn extract(headers: &HeaderMap) -> Result<Self, CustomError> {
        let header = headers
            .get(header::AUTHORIZATION)
            .ok_or(CustomError::Unauthorized)?;

        let value = header.to_str().map_err(|_| CustomError::Unauthorized)?;

        let token = value
            .strip_prefix("Bearer ")
            .ok_or(CustomError::Unauthorized)?;

        if token.is_empty() {
            return Err(CustomError::Unauthorized);
        }

        Ok(Self(token.to_string()))
    }

    /// Hash the token with SHA-256 for storage/comparison.
    pub fn hash(&self) -> AuthTokenHash {
        let mut hasher = Sha256::new();
        hasher.update(self.0.as_bytes());
        AuthTokenHash(hex::encode(hasher.finalize()))
    }

    /// Verify the token against a stored hash using constant-time comparison.
    /// - `None` = no row exists (new secret) → allow access
    /// - `Some(None)` = row exists with NULL hash (legacy) → deny access
    /// - `Some(Some(hash))` = row exists with hash → compare
    pub fn verify(&self, stored_hash: &Option<Option<String>>) -> Result<(), CustomError> {
        match stored_hash {
            Some(Some(expected_hash)) => {
                let provided_hash = self.hash();
                if provided_hash
                    .0
                    .as_bytes()
                    .ct_eq(expected_hash.as_bytes())
                    .into()
                {
                    Ok(())
                } else {
                    Err(CustomError::Forbidden)
                }
            }
            Some(None) => Err(CustomError::Forbidden),
            None => Ok(()),
        }
    }
}
