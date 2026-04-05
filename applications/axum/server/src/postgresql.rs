use sqlx::PgPool;

pub async fn create_db_pool(connection_str: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    Ok(PgPool::connect(connection_str).await?)
}
