use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

/// Create a PostgreSQL connection pool. Panics with a clear message if the
/// database is unreachable, since the service cannot run without it.
pub async fn establish_connection(database_url: &str) -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .unwrap_or_else(|e| panic!("failed to connect to database: {e}"))
}
