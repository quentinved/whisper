use crate::error::CliError;
use aes_gcm_crypto::{derive_auth_token, derive_key, AesGcm};
use tracing::debug;
use whisper_core::{
    services::secret_encryption::SecretEncryption,
    values_object::shared_secret::secret_encrypted::SecretEncrypted,
};

// KDF version bytes — append new versions here when migrating
const KDF_V1_PBKDF2_SHA256_600K: u8 = 0x01;
const CURRENT_KDF_VERSION: u8 = KDF_V1_PBKDF2_SHA256_600K;

pub struct CryptoContext {
    aes: AesGcm,
    auth_token: String,
}

impl CryptoContext {
    pub fn new(passphrase: &str, server_url: &str) -> Result<Self, CliError> {
        debug!(
            "Deriving key (version=0x{:02x}, PBKDF2-SHA256, 600k iterations)...",
            CURRENT_KDF_VERSION
        );
        let key = derive_key_for_version(CURRENT_KDF_VERSION, passphrase, server_url)?;
        let auth_token = derive_auth_token(passphrase, server_url);
        debug!("Key derived");
        Ok(Self {
            aes: AesGcm::new(key),
            auth_token,
        })
    }

    pub fn auth_token(&self) -> &str {
        &self.auth_token
    }

    /// Encrypt plaintext. Payload format: [kdf_version][nonce_12][ciphertext...]
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, CliError> {
        let encrypted =
            self.aes
                .encrypt_secret(plaintext)
                .map_err(|e| CliError::EncryptionFailed {
                    reason: e.to_string(),
                })?;

        let (nonce, cypher) = encrypted.into_parts();
        let mut payload = vec![CURRENT_KDF_VERSION];
        payload.extend(&nonce);
        payload.extend(&cypher);
        Ok(payload)
    }

    /// Decrypt a versioned payload. Reads the KDF version byte, verifies it matches,
    /// then decrypts the rest.
    pub fn decrypt(&self, payload: &[u8]) -> Result<String, CliError> {
        // version(1) + nonce(12) + ciphertext(1+) = 14 bytes minimum
        if payload.len() < 14 {
            return Err(CliError::DecryptionError("Payload too short".to_string()));
        }

        let version = payload[0];
        if version != CURRENT_KDF_VERSION {
            return Err(CliError::UnsupportedKdfVersion(version));
        }

        let nonce: [u8; 12] = payload[1..13]
            .try_into()
            .map_err(|_| CliError::DecryptionError("Invalid nonce length".to_string()))?;
        let cypher = payload[13..].to_vec();
        let encrypted = SecretEncrypted::new(nonce, cypher);

        self.aes
            .decrypt_secret(encrypted)
            .map_err(|e| CliError::DecryptionError(e.to_string()))
    }
}

fn derive_key_for_version(version: u8, passphrase: &str, salt: &str) -> Result<[u8; 32], CliError> {
    match version {
        KDF_V1_PBKDF2_SHA256_600K => Ok(derive_key(passphrase, salt)),
        // Future: KDF_V2_ARGON2 => Ok(argon2_derive_key(passphrase, salt)),
        _ => Err(CliError::UnsupportedKdfVersion(version)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let ctx = CryptoContext::new("test-passphrase", "https://whisper.example.com").unwrap();
        let secret = "my-secret-value";

        let payload = ctx.encrypt(secret).unwrap();

        assert_eq!(payload[0], CURRENT_KDF_VERSION);

        let decrypted = ctx.decrypt(&payload).unwrap();
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let ctx1 = CryptoContext::new("correct-passphrase", "https://whisper.example.com").unwrap();
        let ctx2 = CryptoContext::new("wrong-passphrase", "https://whisper.example.com").unwrap();

        let payload = ctx1.encrypt("secret").unwrap();
        let result = ctx2.decrypt(&payload);

        assert!(result.is_err());
    }

    #[test]
    fn test_payload_too_short() {
        let ctx = CryptoContext::new("pass", "url").unwrap();
        let result = ctx.decrypt(&[0x01; 5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_version_rejected() {
        let ctx = CryptoContext::new("pass", "url").unwrap();
        let mut payload = ctx.encrypt("secret").unwrap();
        payload[0] = 0xFF;
        let result = ctx.decrypt(&payload);
        assert!(matches!(result, Err(CliError::UnsupportedKdfVersion(0xFF))));
    }

    #[test]
    fn test_version_byte_is_first() {
        let ctx = CryptoContext::new("pass", "url").unwrap();
        let payload = ctx.encrypt("hello").unwrap();
        assert_eq!(payload[0], 0x01);
        assert!(payload.len() >= 14); // version + nonce + at least 1 byte ciphertext
    }
}
