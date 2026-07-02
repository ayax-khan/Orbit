mod auth;
mod config;
mod db;
mod device;
mod error;
mod session;
mod state;
mod websocket;

use axum::{
    routing::{delete, get, post},
    Router,
};
use config::Config;
use state::AppState;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // Structured logging (RUST_LOG controls verbosity).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env();
    let bind_address = config.bind_address.clone();

    let db_pool = db::connection::establish_connection(&config.database_url).await;

    // Apply pending migrations automatically on startup.
    if let Err(e) = sqlx::migrate!("./migrations").run(&db_pool).await {
        tracing::error!("failed to run migrations: {e}");
        std::process::exit(1);
    }

    let state = AppState::new(db_pool, config.clone());

    // CORS: restrict to configured origins when provided, otherwise allow any
    // (useful for local development with the unpacked extension).
    let cors = if config.cors_allowed_origins.is_empty() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins = config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect::<Vec<_>>();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let app = Router::new()
        .route("/", get(|| async { "ORBIT Backend is running" }))
        .route("/health", get(|| async { "ok" }))
        // Auth
        .route("/api/v1/auth/register", post(auth::handlers::register))
        .route("/api/v1/auth/login", post(auth::handlers::login))
        .route("/api/v1/auth/refresh", post(auth::handlers::refresh))
        // Devices (bearer-authenticated inside handlers)
        .route("/api/v1/devices/register", post(device::handlers::register_device))
        .route("/api/v1/devices", get(device::handlers::list_devices))
        .route("/api/v1/devices/{device_id}", get(device::handlers::get_device))
        // Sessions
        .route("/api/v1/sessions/create", post(session::handlers::create_session))
        .route("/api/v1/sessions/{session_id}/accept", post(session::handlers::accept_session))
        .route("/api/v1/sessions/{session_id}", delete(session::handlers::end_session))
        // WebRTC signaling (session token itself authorizes the room)
        .route("/ws/{session_id}", get(websocket::handler::ws_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {bind_address}: {e}"));
    tracing::info!("ORBIT backend listening on http://{bind_address}");
    axum::serve(listener, app).await.unwrap();
}