use std::str::from_utf8;

use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

use whisper_core::{
    services::secret_encryption::{SecretEncryption, SecretEncryptionError},
    values_object::shared_secret::secret_encrypted::SecretEncrypted,
};

pub struct AesGcm {
    cipher: Aes256Gcm,
}

impl AesGcm {
    pub fn new(key: impl Into<Key<Aes256Gcm>>) -> Self {
        let key = key.into();
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }
}

impl SecretEncryption for AesGcm {
    fn encrypt_secret(&self, secret: &str) -> Result<SecretEncrypted, SecretEncryptionError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let nonce: &[u8; 12] =
            nonce
                .as_slice()
                .try_into()
                .map_err(|_| SecretEncryptionError::InternalError {
                    reason: "nonce length mismatch".to_string(),
                })?;
        let ciphertext = self
            .cipher
            .encrypt(nonce.into(), secret.as_bytes())
            .map_err(|err| SecretEncryptionError::InternalError {
                reason: err.to_string(),
            })?;
        Ok(SecretEncrypted::new(*nonce, ciphertext))
    }

    fn decrypt_secret(
        &self,
        encrypted_secret: SecretEncrypted,
    ) -> Result<String, SecretEncryptionError> {
        let secret = self
            .cipher
            .decrypt(encrypted_secret.nonce().into(), encrypted_secret.cypher())
            .map_err(|err| SecretEncryptionError::InternalError {
                reason: err.to_string(),
            })?;
        let secret = from_utf8(&secret).map_err(|_| SecretEncryptionError::InternalError {
            reason: "invalid UTF-8 in decrypted secret".to_string(),
        })?;
        Ok(secret.to_string())
    }
}

pub fn derive_key(passphrase: &str, salt: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt.as_bytes(), 600_000, &mut key);
    key
}

pub fn derive_auth_token(passphrase: &str, server_url: &str) -> String {
    let auth_salt = format!("{}:auth", server_url);
    let key = derive_key(passphrase, &auth_salt);
    hex::encode(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("password123")]
    #[case("another_password!")]
    #[case("复杂密码")]
    #[case("p@$$w0rd!")]
    fn try_encrypt_decrypt(#[case] secret: &str) {
        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
        let aes_gcm = AesGcm::new(*key);
        let encrypted_secret = aes_gcm.encrypt_secret(secret).unwrap();
        let decrypted_secret = aes_gcm.decrypt_secret(encrypted_secret).unwrap();
        assert_eq!(decrypted_secret, secret);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let key1 = derive_key("my-passphrase", "https://whisper.example.com");
        let key2 = derive_key("my-passphrase", "https://whisper.example.com");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_passphrase_different_key() {
        let key1 = derive_key("passphrase-a", "https://whisper.example.com");
        let key2 = derive_key("passphrase-b", "https://whisper.example.com");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_salt_different_key() {
        let key1 = derive_key("same-passphrase", "https://instance-a.com");
        let key2 = derive_key("same-passphrase", "https://instance-b.com");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_works_with_aes_gcm() {
        let key = derive_key("my-passphrase", "https://whisper.example.com");
        let aes = AesGcm::new(key);
        let encrypted = aes.encrypt_secret("hello world").unwrap();
        let decrypted = aes.decrypt_secret(encrypted).unwrap();
        assert_eq!(decrypted, "hello world");
    }
}
