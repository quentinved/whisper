use crate::values_object::shared_secret::{
    secret_encrypted::SecretEncrypted, secret_expiration::SecretExpiration, secret_id::SecretId,
};

#[derive(Clone, Debug)]
pub struct SharedSecret {
    id: SecretId,
    encrypted_secret: SecretEncrypted,
    expiration: SecretExpiration,
    self_destruct: bool,
}

impl SharedSecret {
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
}
