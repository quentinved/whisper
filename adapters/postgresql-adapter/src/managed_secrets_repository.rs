use crate::models::{managed_secret::PostgreSQLManagedSecret, ModelConversionError};
use sqlx::PgPool;
use whisper_core::{
    contracts::repositories::managed_secret_repository::{
        ManagedSecretRepository, ManagedSecretRepositoryError,
    },
    entities::managed_secret::ManagedSecret,
    values_object::shared_secret::secret_id::SecretId,
};

#[derive(Clone)]
pub struct PostgreSQLManagedSecretsRepository {
    pub pool: PgPool,
}

impl PostgreSQLManagedSecretsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ManagedSecretRepository for PostgreSQLManagedSecretsRepository {
    async fn upsert(&self, secret: ManagedSecret) -> Result<bool, ManagedSecretRepositoryError> {
        let pg_secret: PostgreSQLManagedSecret = secret.into();
        let query = "
            INSERT INTO managed_secrets (id, payload, created_at, updated_at, auth_token_hash)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET payload = $2, updated_at = $4
            RETURNING (xmax = 0) AS is_insert
        ";

        let is_insert: bool = sqlx::query_scalar(query)
            .bind(&pg_secret.id)
            .bind(&pg_secret.payload)
            .bind(pg_secret.created_at)
            .bind(pg_secret.updated_at)
            .bind(&pg_secret.auth_token_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                tracing::error!("Database error during upsert: {:?}", err);
                ManagedSecretRepositoryError::DatabaseError("Failed to upsert secret".to_string())
            })?;

        Ok(is_insert)
    }

    async fn get_by_id(
        &self,
        id: &SecretId,
    ) -> Result<Option<ManagedSecret>, ManagedSecretRepositoryError> {
        let query = "SELECT * FROM managed_secrets WHERE id = $1";
        let pg_secret = sqlx::query_as::<_, PostgreSQLManagedSecret>(query)
            .bind(id.value().to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                tracing::error!("Database error during get_by_id: {:?}", err);
                ManagedSecretRepositoryError::DatabaseError("Failed to retrieve secret".to_string())
            })?;

        match pg_secret {
            Some(pg) => {
                let secret: ManagedSecret =
                    pg.try_into().map_err(|err: ModelConversionError| {
                        ManagedSecretRepositoryError::ServiceError(err.to_string())
                    })?;
                Ok(Some(secret))
            }
            None => Ok(None),
        }
    }

    async fn pull_by_id(
        &self,
        id: &SecretId,
    ) -> Result<Option<ManagedSecret>, ManagedSecretRepositoryError> {
        let query = "
            UPDATE managed_secrets
            SET last_pulled_at = NOW()
            WHERE id = $1
            RETURNING *
        ";
        let pg_secret = sqlx::query_as::<_, PostgreSQLManagedSecret>(query)
            .bind(id.value().to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                tracing::error!("Database error during pull_by_id: {:?}", err);
                ManagedSecretRepositoryError::DatabaseError("Failed to pull secret".to_string())
            })?;

        match pg_secret {
            Some(pg) => {
                let secret: ManagedSecret =
                    pg.try_into().map_err(|err: ModelConversionError| {
                        ManagedSecretRepositoryError::ServiceError(err.to_string())
                    })?;
                Ok(Some(secret))
            }
            None => Ok(None),
        }
    }

    async fn get_auth_token_hash(
        &self,
        id: &SecretId,
    ) -> Result<Option<Option<String>>, ManagedSecretRepositoryError> {
        let query = "SELECT auth_token_hash FROM managed_secrets WHERE id = $1";
        let result: Option<(Option<String>,)> = sqlx::query_as(query)
            .bind(id.value().to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                tracing::error!("Database error during get_auth_token_hash: {:?}", err);
                ManagedSecretRepositoryError::DatabaseError(
                    "Failed to retrieve auth token hash".to_string(),
                )
            })?;
        Ok(result.map(|(hash,)| hash))
    }

    async fn delete_by_id(&self, id: &SecretId) -> Result<(), ManagedSecretRepositoryError> {
        let query = "DELETE FROM managed_secrets WHERE id = $1";
        sqlx::query(query)
            .bind(id.value().to_string())
            .execute(&self.pool)
            .await
            .map_err(|err| {
                tracing::error!("Database error during delete_by_id: {:?}", err);
                ManagedSecretRepositoryError::DatabaseError("Failed to delete secret".to_string())
            })?;
        Ok(())
    }
}
