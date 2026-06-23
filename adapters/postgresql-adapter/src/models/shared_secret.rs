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
    pub client_encrypted: bool,
}

impl From<SharedSecret> for PostgreSQLSharedSecret {
    fn from(secret: SharedSecret) -> Self {
        let id = secret.id().to_string();
        let expiration = secret.expiration().value();
        let self_destruct = secret.self_destruct();
        let client_encrypted = secret.client_encrypted();
        let (nonce, cypher) = secret.encrypted_secret().into_parts();
        Self {
            id,
            cypher,
            nonce,
            expiration,
            self_destruct,
            client_encrypted,
        }
    }
}

impl TryFrom<PostgreSQLSharedSecret> for SharedSecret {
    type Error = ModelConversionError;

    fn try_from(pg: PostgreSQLSharedSecret) -> Result<Self, Self::Error> {
        let id = SecretId::try_from(pg.id.as_str())?;
        let encrypted_secret = SecretEncrypted::new(pg.nonce, pg.cypher);
        let expiration = SecretExpiration::from_datetime(pg.expiration);
        Ok(if pg.client_encrypted {
            SharedSecret::new_client_encrypted(id, encrypted_secret, expiration, pg.self_destruct)
        } else {
            SharedSecret::new(id, encrypted_secret, expiration, pg.self_destruct)
        })
    }
}
