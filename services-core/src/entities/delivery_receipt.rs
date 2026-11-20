use crate::values_object::shared_secret::{
    secret_expiration::SecretExpiration, secret_id::SecretId, status_token::StatusToken,
};
use chrono::{DateTime, Utc};

/// How long a receipt outlives its secret, so the sender can still see
/// "opened" or "expired, never opened" after the secret itself is gone.
pub const RECEIPT_RETENTION_DAYS: i64 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    Waiting,
    Opened { opened_at: DateTime<Utc> },
    Expired,
}

/// Records when a tracked secret was first revealed. Outlives the secret row,
/// which is deleted on a self-destruct read.
#[derive(Debug, Clone)]
pub struct DeliveryReceipt {
    status_token: StatusToken,
    secret_id: SecretId,
    expiration: SecretExpiration,
    opened_at: Option<DateTime<Utc>>,
}

impl DeliveryReceipt {
    pub fn new(
        status_token: StatusToken,
        secret_id: SecretId,
        expiration: SecretExpiration,
        opened_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            status_token,
            secret_id,
            expiration,
            opened_at,
        }
    }

    pub fn status_token(&self) -> StatusToken {
        self.status_token
    }

    pub fn secret_id(&self) -> SecretId {
        self.secret_id
    }

    pub fn expiration(&self) -> SecretExpiration {
        self.expiration
    }

    pub fn opened_at(&self) -> Option<DateTime<Utc>> {
        self.opened_at
    }

    /// Opened wins over expired: a secret read just before it expired still
    /// reports when it was opened.
    pub fn status(&self) -> DeliveryStatus {
        match self.opened_at {
            Some(opened_at) => DeliveryStatus::Opened { opened_at },
            None if self.expiration.is_expired() => DeliveryStatus::Expired,
            None => DeliveryStatus::Waiting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn receipt(expires_in: Duration, opened_at: Option<DateTime<Utc>>) -> DeliveryReceipt {
        DeliveryReceipt::new(
            StatusToken::generate(),
            SecretId::generate(),
            SecretExpiration::from_datetime(Utc::now() + expires_in),
            opened_at,
        )
    }

    #[test]
    fn unopened_live_secret_is_waiting() {
        assert_eq!(
            receipt(Duration::hours(1), None).status(),
            DeliveryStatus::Waiting
        );
    }

    #[test]
    fn unopened_expired_secret_is_expired() {
        assert_eq!(
            receipt(Duration::hours(-1), None).status(),
            DeliveryStatus::Expired
        );
    }

    #[test]
    fn opened_secret_reports_open_time_even_after_expiry() {
        let opened_at = Utc::now() - Duration::hours(2);
        assert_eq!(
            receipt(Duration::hours(-1), Some(opened_at)).status(),
            DeliveryStatus::Opened { opened_at }
        );
    }
}
