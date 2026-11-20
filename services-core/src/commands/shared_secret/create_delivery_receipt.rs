use crate::{
    contracts::repositories::shared_secret_repository::SharedSecretRepository,
    entities::delivery_receipt::DeliveryReceipt,
    values_object::shared_secret::{
        secret_expiration::SecretExpiration, secret_id::SecretId, status_token::StatusToken,
    },
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateDeliveryReceiptError {
    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

/// Starts tracking a secret's delivery and hands the sender its status token.
pub struct CreateDeliveryReceipt {
    secret_id: SecretId,
    expiration: SecretExpiration,
}

impl CreateDeliveryReceipt {
    pub fn new(secret_id: SecretId, expiration: SecretExpiration) -> Self {
        Self {
            secret_id,
            expiration,
        }
    }

    pub async fn handle(
        &self,
        shared_secret_repository: &impl SharedSecretRepository,
    ) -> Result<StatusToken, CreateDeliveryReceiptError> {
        let status_token = StatusToken::generate();
        let receipt = DeliveryReceipt::new(status_token, self.secret_id, self.expiration, None);
        shared_secret_repository
            .save_receipt(receipt)
            .await
            .map_err(|err| CreateDeliveryReceiptError::InternalError {
                reason: err.to_string(),
            })?;
        Ok(status_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::shared_secret::test_utils::mocks::MockSharedSecretRepository;
    use crate::entities::delivery_receipt::DeliveryStatus;
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn creates_a_waiting_receipt_reachable_by_its_token() {
        let repository = MockSharedSecretRepository::new();
        let secret_id = SecretId::generate();
        let expiration = SecretExpiration::from_datetime(Utc::now() + Duration::hours(1));

        let token = CreateDeliveryReceipt::new(secret_id, expiration)
            .handle(&repository)
            .await
            .unwrap();

        let receipt = repository.get_receipt(&token).await.unwrap().unwrap();
        assert_eq!(receipt.secret_id(), secret_id);
        assert_eq!(receipt.status(), DeliveryStatus::Waiting);
    }
}
