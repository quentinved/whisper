use chrono::{DateTime, Duration, TimeZone, Utc};
use thiserror::Error;

const MAX_EXPIRATION_DAYS: i64 = 7;

#[derive(Clone, Copy, Debug)]
pub struct SecretExpiration {
    value: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum SecretExpirationError {
    #[error("Invalid expiration: {reason}")]
    InvalidExpiration { reason: String },
}

impl SecretExpiration {
    /// Reconstruct from a stored timestamp
    pub fn from_datetime(value: DateTime<Utc>) -> Self {
        Self { value }
    }

    pub fn value(&self) -> DateTime<Utc> {
        self.value
    }

    pub fn is_expired(&self) -> bool {
        self.value < Utc::now()
    }
}

impl TryFrom<i64> for SecretExpiration {
    type Error = SecretExpirationError;

    fn try_from(timestamp: i64) -> Result<Self, Self::Error> {
        let value = match Utc.timestamp_opt(timestamp, 0) {
            chrono::offset::LocalResult::Single(value) => value,
            chrono::offset::LocalResult::Ambiguous(earlier, latest) => {
                return Err(SecretExpirationError::InvalidExpiration {
                    reason: format!("Ambiguous expiration: {} or {}", earlier, latest),
                })
            }
            chrono::offset::LocalResult::None => {
                return Err(SecretExpirationError::InvalidExpiration {
                    reason: "Invalid expiration".to_string(),
                })
            }
        };
        if value < Utc::now() {
            return Err(SecretExpirationError::InvalidExpiration {
                reason: "Expiration is in the past".to_string(),
            });
        }
        let max_expiration = Utc::now() + Duration::days(MAX_EXPIRATION_DAYS);
        if value > max_expiration {
            return Err(SecretExpirationError::InvalidExpiration {
                reason: format!("Expiration cannot exceed {} days", MAX_EXPIRATION_DAYS),
            });
        }
        Ok(Self { value })
    }
}
