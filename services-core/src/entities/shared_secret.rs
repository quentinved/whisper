use crate::values_object::shared_secret::{
    secret_encrypted::SecretEncrypted, secret_expiration::SecretExpiration, secret_id::SecretId,
};

#[derive(Clone, Debug)]
pub struct SharedSecret {
    id: SecretId,
    encrypted_secret: SecretEncrypted,
    expiration: SecretExpiration,
    self_destruct: bool,
    client_encrypted: bool,
}

impl SharedSecret {
    /// Server-encrypted secret: encrypted at rest with the server-held key.
    pub fn new(
        id: SecretId,
        encrypted_secret: SecretEncrypted,
        expiration: SecretExpiration,
        self_destruct: bool,
    ) -> Self {
        Self {
            id,
            encrypted_secret,
            expiration,
            self_destruct,
            client_encrypted: false,
        }
    }

    /// Client-encrypted (zero-knowledge) secret: nonce/cypher were produced by the
    /// sender's device; the server holds no key for them.
    pub fn new_client_encrypted(
        id: SecretId,
        encrypted_secret: SecretEncrypted,
        expiration: SecretExpiration,
        self_destruct: bool,
    ) -> Self {
        Self {
            id,
            encrypted_secret,
            expiration,
            self_destruct,
            client_encrypted: true,
        }
    }

    pub fn id(&self) -> SecretId {
        self.id
    }

    pub fn encrypted_secret(self) -> SecretEncrypted {
        self.encrypted_secret
    }

    pub fn expiration(&self) -> SecretExpiration {
        self.expiration
    }

    pub fn self_destruct(&self) -> bool {
        self.self_destruct
    }

    pub fn client_encrypted(&self) -> bool {
        self.client_encrypted
    }
}
