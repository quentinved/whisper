use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use whisper_core::{
    entities::delivery_receipt::DeliveryReceipt,
    values_object::shared_secret::{
        secret_expiration::SecretExpiration, secret_id::SecretId, status_token::StatusToken,
    },
};

use super::ModelConversionError;

#[derive(Debug, FromRow)]
pub struct PostgreSQLDeliveryReceipt {
    pub status_token: String,
    pub secret_id: String,
    pub expiration: DateTime<Utc>,
    pub opened_at: Option<DateTime<Utc>>,
}

impl TryFrom<PostgreSQLDeliveryReceipt> for DeliveryReceipt {
    type Error = ModelConversionError;

    fn try_from(pg: PostgreSQLDeliveryReceipt) -> Result<Self, Self::Error> {
        Ok(DeliveryReceipt::new(
            StatusToken::try_from(pg.status_token.as_str())?,
            SecretId::try_from(pg.secret_id.as_str())?,
            SecretExpiration::from_datetime(pg.expiration),
            pg.opened_at,
        ))
    }
}
