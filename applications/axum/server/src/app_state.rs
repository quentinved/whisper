use aes_gcm_crypto::AesGcm;
use sqlx::PgPool;
use std::sync::Arc;
use whisper_core::{
    contracts::repositories::managed_secret_repository::ManagedSecretRepository,
    contracts::repositories::shared_secret_repository::SharedSecretRepository,
    services::secret_encryption::SecretEncryption,
};
use whisper_postgresql::managed_secrets_repository::PostgreSQLManagedSecretsRepository;
use whisper_postgresql::shared_secrets_repository::PostgreSQLSharedSecretsRepository;

use crate::analytics::AnalyticsTracker;

#[derive(Debug, Clone)]
pub struct AppState {
    pool: PgPool,
    key: [u8; 32],
    url: String,
    slack_signing_secret: Option<String>,
    analytics: Arc<dyn AnalyticsTracker>,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        key: [u8; 32],
        url: String,
        slack_signing_secret: Option<String>,
        analytics: Arc<dyn AnalyticsTracker>,
    ) -> Self {
        Self {
            pool,
            key,
            url,
            slack_signing_secret,
            analytics,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn slack_signing_secret(&self) -> Option<&str> {
        self.slack_signing_secret.as_deref()
    }

    pub fn analytics(&self) -> &Arc<dyn AnalyticsTracker> {
        &self.analytics
    }

    pub fn shared_secret_repository(&self) -> impl SharedSecretRepository {
        PostgreSQLSharedSecretsRepository::new(self.pool.clone())
    }

    pub fn aes_gcm(&self) -> impl SecretEncryption {
        AesGcm::new(self.key)
    }

    pub fn managed_secret_repository(&self) -> impl ManagedSecretRepository {
        PostgreSQLManagedSecretsRepository::new(self.pool.clone())
    }
}
