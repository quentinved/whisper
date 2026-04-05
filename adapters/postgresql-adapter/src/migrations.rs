use sqlx::PgPool;

pub async fn apply_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}
