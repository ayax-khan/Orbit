use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;

pub async fn establish_connection() -> sqlx::PgPool {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Can't connect to database")
}
