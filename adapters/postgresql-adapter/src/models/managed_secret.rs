use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use whisper_core::{
    entities::managed_secret::ManagedSecret, values_object::shared_secret::secret_id::SecretId,
};

use super::ModelConversionError;

#[derive(Debug, FromRow)]
pub struct PostgreSQLManagedSecret {
    pub id: String,
    pub payload: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_pulled_at: Option<DateTime<Utc>>,
    pub auth_token_hash: Option<String>,
}

impl From<ManagedSecret> for PostgreSQLManagedSecret {
    fn from(secret: ManagedSecret) -> Self {
        let id = secret.id().to_string();
        let created_at = secret.created_at();
        let updated_at = secret.updated_at();
        let last_pulled_at = secret.last_pulled_at();
        let auth_token_hash = secret.auth_token_hash().map(String::from);
        Self {
            id,
            payload: secret.into_payload(),
            created_at,
            updated_at,
            last_pulled_at,
            auth_token_hash,
        }
    }
}

impl TryFrom<PostgreSQLManagedSecret> for ManagedSecret {
    type Error = ModelConversionError;

    fn try_from(pg: PostgreSQLManagedSecret) -> Result<Self, Self::Error> {
        let id = SecretId::try_from(pg.id.as_str())?;
        Ok(ManagedSecret::from_persisted(
            id,
            pg.payload,
            pg.created_at,
            pg.updated_at,
            pg.last_pulled_at,
            pg.auth_token_hash,
        ))
    }
}
