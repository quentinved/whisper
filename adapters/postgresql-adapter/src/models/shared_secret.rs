use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use whisper_core::{
    entities::shared_secret::SharedSecret,
    values_object::shared_secret::{
        secret_encrypted::SecretEncrypted, secret_expiration::SecretExpiration, secret_id::SecretId,
    },
};

use super::ModelConversionError;

#[derive(Debug, FromRow)]
pub struct PostgreSQLSharedSecret {
    pub id: String,
    pub cypher: Vec<u8>,
    pub nonce: [u8; 12],
    pub expiration: DateTime<Utc>,
    pub self_destruct: bool,
}

impl From<SharedSecret> for PostgreSQLSharedSecret {
    fn from(secret: SharedSecret) -> Self {
        let id = secret.id().to_string();
        let expiration = secret.expiration().value();
        let self_destruct = secret.self_destruct();
        let (nonce, cypher) = secret.encrypted_secret().into_parts();
        Self {
            id,
            cypher,
            nonce,
            expiration,
            self_destruct,
        }
    }
}

impl TryFrom<PostgreSQLSharedSecret> for SharedSecret {
    type Error = ModelConversionError;

    fn try_from(pg: PostgreSQLSharedSecret) -> Result<Self, Self::Error> {
        let id = SecretId::try_from(pg.id.as_str())?;
        let encrypted_secret = SecretEncrypted::new(pg.nonce, pg.cypher);
        Ok(SharedSecret::new(
            id,
            encrypted_secret,
            SecretExpiration::from_datetime(pg.expiration),
            pg.self_destruct,
        ))
    }
}
