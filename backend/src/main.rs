mod db;
mod auth;
mod websocket;
mod middleware;

use axum::{routing::{get, post}, Router, middleware as axum_middleware};
use std::sync::Arc;

struct AppState {
    db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    let db_pool = db::connection::establish_connection().await;
    let shared_state = Arc::new(AppState { db: db_pool });

    // 1. Public Routes
    let public_routes = Router::new()
        .route("/", get(|| async { "ORBIT Backend is running" }))
        .route("/api/v1/auth/register", post(auth::handlers::register))
        .route("/api/v1/auth/login", post(auth::handlers::login));

    // 2. Protected Routes (Auth Middleware applied here)
    let protected_routes = Router::new()
        .route("/api/v1/sessions/create", post(auth::handlers::register)) // Should be session create
        .route("/ws/{session_id}", get(websocket::handler::ws_handler))
        .route_layer(axum_middleware::from_fn(middleware::auth::auth_middleware));

    // 3. Merge
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server running on http://127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();
}