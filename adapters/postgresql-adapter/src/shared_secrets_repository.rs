use crate::models::{shared_secret::PostgreSQLSharedSecret, ModelConversionError};
use sqlx::PgPool;
use whisper_core::{
    contracts::repositories::shared_secret_repository::{
        SharedSecretRepository, SharedSecretRepositoryError,
    },
    entities::shared_secret::SharedSecret,
    values_object::shared_secret::secret_id::SecretId,
};

#[derive(Clone)]
pub struct PostgreSQLSharedSecretsRepository {
    pub pool: PgPool,
}

impl PostgreSQLSharedSecretsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SharedSecretRepository for PostgreSQLSharedSecretsRepository {
    async fn save(
        &self,
        shared_secret: SharedSecret,
    ) -> Result<SecretId, SharedSecretRepositoryError> {
        let secret_id = shared_secret.id();
        let postgresql_secret: PostgreSQLSharedSecret = shared_secret.into();
        tracing::debug!(secret_id = %secret_id, "saving secret to database");
        let query = "
            INSERT INTO secrets (id, cypher, nonce, expiration, self_destruct)
            VALUES ($1, $2, $3, $4, $5)
        ";

        if let Err(error) = sqlx::query(query)
            .bind(postgresql_secret.id)
            .bind(postgresql_secret.cypher)
            .bind(postgresql_secret.nonce.as_slice())
            .bind(postgresql_secret.expiration)
            .bind(postgresql_secret.self_destruct)
            .execute(&self.pool)
            .await
        {
            tracing::error!("Database error during save: {:?}", error);
            return Err(SharedSecretRepositoryError::DatabaseError(
                "Failed to save secret".to_string(),
            ));
        };

        Ok(secret_id)
    }

    async fn get_by_id(
        &self,
        id: &SecretId,
    ) -> Result<Option<SharedSecret>, SharedSecretRepositoryError> {
        let id_str = id.value().to_string();

        // Atomically delete self-destruct secrets on read
        let delete_query = "
            DELETE FROM secrets
            WHERE id = $1 AND expiration >= NOW() AND self_destruct = true
            RETURNING *
        ";
        let deleted = sqlx::query_as::<_, PostgreSQLSharedSecret>(delete_query)
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(reason) => {
                    tracing::error!("Database error during get_by_id (delete): {:?}", reason);
                    SharedSecretRepositoryError::DatabaseError(
                        "Failed to retrieve secret".to_string(),
                    )
                }
                _ => SharedSecretRepositoryError::ServiceError(err.to_string()),
            })?;

        if let Some(pg_secret) = deleted {
            let secret: SharedSecret =
                pg_secret.try_into().map_err(|err: ModelConversionError| {
                    SharedSecretRepositoryError::ServiceError(err.to_string())
                })?;
            return Ok(Some(secret));
        }

        // Non-self-destruct: plain SELECT
        let select_query = "
            SELECT *
            FROM secrets
            WHERE id = $1 AND expiration >= NOW()
        ";
        let pg_secret = sqlx::query_as::<_, PostgreSQLSharedSecret>(select_query)
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(reason) => {
                    tracing::error!("Database error during get_by_id (select): {:?}", reason);
                    SharedSecretRepositoryError::DatabaseError(
                        "Failed to retrieve secret".to_string(),
                    )
                }
                _ => SharedSecretRepositoryError::ServiceError(err.to_string()),
            })?;

        match pg_secret {
            Some(pg) => {
                let secret: SharedSecret = pg.try_into().map_err(|err: ModelConversionError| {
                    SharedSecretRepositoryError::ServiceError(err.to_string())
                })?;
                Ok(Some(secret))
            }
            None => Ok(None),
        }
    }

    async fn delete_by_id(&self, id: &SecretId) -> Result<(), SharedSecretRepositoryError> {
        let query = "
        DELETE FROM secrets
        WHERE id = $1
    ";
        let result = sqlx::query(query)
            .bind(id.value().to_string())
            .execute(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref db_err) => {
                    tracing::error!("Database error during delete_by_id: {:?}", db_err);
                    SharedSecretRepositoryError::DatabaseError(
                        "Failed to delete secret".to_string(),
                    )
                }
                _ => SharedSecretRepositoryError::ServiceError(err.to_string()),
            })?;

        if result.rows_affected() == 0 {
            return Err(SharedSecretRepositoryError::NotFound);
        }

        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<SharedSecret>, SharedSecretRepositoryError> {
        let query = "
        SELECT *
        FROM secrets
    ";
        let postgre_shared_secrets = sqlx::query_as::<_, PostgreSQLSharedSecret>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref db_err) => {
                    tracing::error!("Database error during get_all: {:?}", db_err);
                    SharedSecretRepositoryError::DatabaseError(
                        "Failed to retrieve secrets".to_string(),
                    )
                }
                _ => SharedSecretRepositoryError::ServiceError(err.to_string()),
            })?;

        let shared_secrets = postgre_shared_secrets
            .into_iter()
            .map(|postgre_shared_secret| {
                postgre_shared_secret
                    .try_into()
                    .map_err(|err: ModelConversionError| {
                        SharedSecretRepositoryError::ServiceError(err.to_string())
                    })
            })
            .collect::<Result<Vec<SharedSecret>, SharedSecretRepositoryError>>()?;

        Ok(shared_secrets)
    }

    async fn get_all_expired(&self) -> Result<Vec<SharedSecret>, SharedSecretRepositoryError> {
        let query = "
        SELECT *
        FROM secrets
        WHERE expiration < NOW();
    ";
        let postgre_shared_secrets = sqlx::query_as::<_, PostgreSQLSharedSecret>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref db_err) => {
                    tracing::error!("Database error during get_all_expired: {:?}", db_err);
                    SharedSecretRepositoryError::DatabaseError(
                        "Failed to retrieve expired secrets".to_string(),
                    )
                }
                _ => SharedSecretRepositoryError::ServiceError(err.to_string()),
            })?;

        let shared_secrets = postgre_shared_secrets
            .into_iter()
            .map(|postgre_shared_secret| {
                postgre_shared_secret
                    .try_into()
                    .map_err(|err: ModelConversionError| {
                        SharedSecretRepositoryError::ServiceError(err.to_string())
                    })
            })
            .collect::<Result<Vec<SharedSecret>, SharedSecretRepositoryError>>()?;

        Ok(shared_secrets)
    }

    async fn delete_all_expired(&self) -> Result<u64, SharedSecretRepositoryError> {
        let query = "DELETE FROM secrets WHERE expiration < NOW()";
        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref db_err) => {
                    tracing::error!("Database error during delete_all_expired: {:?}", db_err);
                    SharedSecretRepositoryError::DatabaseError(
                        "Failed to delete expired secrets".to_string(),
                    )
                }
                _ => SharedSecretRepositoryError::ServiceError(err.to_string()),
            })?;
        Ok(result.rows_affected())
    }
}
