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

/// Encrypts a one-time secret with a fresh random 256-bit key (zero-knowledge
/// ephemeral sharing). Returns `(key_b64url, payload)` where payload is
/// `nonce[12] ‖ ciphertext` — the format shared by the web UI, CLI, Slack and
/// Discord, so links are interoperable across all surfaces.
pub fn encrypt_ephemeral(plaintext: &str) -> Result<(String, Vec<u8>), SecretEncryptionError> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let aes = AesGcm::new(key);
    let encrypted = aes.encrypt_secret(plaintext)?;
    let (nonce, cypher) = encrypted.into_parts();
    let mut payload = Vec::with_capacity(nonce.len() + cypher.len());
    payload.extend_from_slice(&nonce);
    payload.extend(cypher);
    Ok((base64_url::encode(key.as_slice()), payload))
}

/// Decrypts an ephemeral payload (`nonce[12] ‖ ciphertext`) using the key
/// carried in a link's `#k=` fragment.
pub fn decrypt_ephemeral(
    key_b64url: &str,
    payload: &[u8],
) -> Result<String, SecretEncryptionError> {
    let key_bytes =
        base64_url::decode(key_b64url).map_err(|_| SecretEncryptionError::InternalError {
            reason: "invalid base64url key".to_string(),
        })?;
    let key: [u8; 32] =
        key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| SecretEncryptionError::InternalError {
                reason: "key must be 32 bytes".to_string(),
            })?;
    if payload.len() < 13 {
        return Err(SecretEncryptionError::InternalError {
            reason: "payload too short".to_string(),
        });
    }
    let nonce: [u8; 12] = payload[..12].try_into().expect("12-byte slice");
    let encrypted = SecretEncrypted::new(nonce, payload[12..].to_vec());
    AesGcm::new(key).decrypt_secret(encrypted)
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
    fn ephemeral_payload_is_nonce_plus_ciphertext() {
        let (_key, payload) = encrypt_ephemeral("x").unwrap();
        // 12-byte nonce + 1 byte plaintext + 16-byte GCM tag
        assert_eq!(payload.len(), 12 + 1 + 16);
    }

    #[test]
    fn ephemeral_key_is_32_bytes_unpadded_b64url() {
        let (key_b64, _payload) = encrypt_ephemeral("x").unwrap();
        let key = base64_url::decode(&key_b64).unwrap();
        assert_eq!(key.len(), 32);
        assert!(!key_b64.contains('='), "must be unpadded base64url");
    }

    /// Known-answer vector generated with browser WebCrypto (Node crypto.subtle):
    /// locks cross-surface compatibility of the wire format permanently.
    ///
    /// Regenerate with:
    ///   node -e '(async()=>{const k=await crypto.subtle.generateKey({name:"AES-GCM",length:256},true,["encrypt"]);const n=crypto.getRandomValues(new Uint8Array(12));const c=new Uint8Array(await crypto.subtle.encrypt({name:"AES-GCM",iv:n},k,new TextEncoder().encode("cross-surface-fixture")));const r=new Uint8Array(await crypto.subtle.exportKey("raw",k));const p=new Uint8Array(12+c.length);p.set(n);p.set(c,12);const b=x=>Buffer.from(x).toString("base64url");console.log(b(r),b(p))})()'
    #[test]
    fn decrypts_webcrypto_generated_payload() {
        let key = "YwVK0LTYDm7yxil3CKnKiU8-Mvsegmf8mxr8In4XmEI";
        let payload = base64_url::decode(
            "Pxt-RW_ccCkmgacnuzz0PoBFwT6bBGqvHjYz8z-Il0An1MspmTijsFFU51iKt0F9IA",
        )
        .unwrap();
        assert_eq!(
            decrypt_ephemeral(key, &payload).unwrap(),
            "cross-surface-fixture"
        );
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
