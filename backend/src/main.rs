mod db;
mod auth;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;

struct AppState {
    db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    let db_pool = db::connection::establish_connection().await;
    let shared_state = Arc::new(AppState { db: db_pool });

    let app = Router::new()
        .route("/", get(|| async { "ORBIT Backend is running" }))
        .route("/api/v1/auth/register", post(auth::handlers::register))
        .route("/api/v1/auth/login", post(auth::handlers::login))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server running on http://127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();
}