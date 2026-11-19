use crate::{
    contracts::repositories::shared_secret_repository::SharedSecretRepository,
    entities::delivery_receipt::DeliveryReceipt,
    values_object::shared_secret::status_token::StatusToken,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetDeliveryReceiptError {
    #[error("Internal Error: {reason}")]
    InternalError { reason: String },
}

/// Reads a secret's delivery receipt. Never touches the secret itself.
pub struct GetDeliveryReceipt {
    status_token: StatusToken,
}

impl GetDeliveryReceipt {
    pub fn new(status_token: StatusToken) -> Self {
        Self { status_token }
    }

    pub async fn handle(
        &self,
        shared_secret_repository: &impl SharedSecretRepository,
    ) -> Result<Option<DeliveryReceipt>, GetDeliveryReceiptError> {
        shared_secret_repository
            .get_receipt(&self.status_token)
            .await
            .map_err(|err| GetDeliveryReceiptError::InternalError {
                reason: err.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::shared_secret::test_utils::mocks::MockSharedSecretRepository;

    #[tokio::test]
    async fn unknown_token_returns_none() {
        let repository = MockSharedSecretRepository::new();
        let result = GetDeliveryReceipt::new(StatusToken::generate())
            .handle(&repository)
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
