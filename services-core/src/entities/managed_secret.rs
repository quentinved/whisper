use chrono::{DateTime, Utc};

use crate::values_object::shared_secret::secret_id::SecretId;

#[derive(Clone, Debug)]
pub struct ManagedSecret {
    id: SecretId,
    payload: Vec<u8>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_pulled_at: Option<DateTime<Utc>>,
    auth_token_hash: Option<String>,
}

impl ManagedSecret {
    pub fn new(id: SecretId, payload: Vec<u8>, auth_token_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            payload,
            created_at: now,
            updated_at: now,
            last_pulled_at: None,
            auth_token_hash: Some(auth_token_hash),
        }
    }

    /// Reconstruct from DB
    pub fn from_persisted(
        id: SecretId,
        payload: Vec<u8>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        last_pulled_at: Option<DateTime<Utc>>,
        auth_token_hash: Option<String>,
    ) -> Self {
        Self {
            id,
            payload,
            created_at,
            updated_at,
            last_pulled_at,
            auth_token_hash,
        }
    }

    pub fn id(&self) -> SecretId {
        self.id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn last_pulled_at(&self) -> Option<DateTime<Utc>> {
        self.last_pulled_at
    }

    pub fn set_last_pulled_at(&mut self, at: DateTime<Utc>) {
        self.last_pulled_at = Some(at);
    }

    pub fn auth_token_hash(&self) -> Option<&str> {
        self.auth_token_hash.as_deref()
    }
}
