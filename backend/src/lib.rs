pub mod auth;
pub mod config;
pub mod contacts;
pub mod db;
pub mod device;
pub mod error;
pub mod session;
pub mod state;
pub mod utils;
pub mod websocket;

use axum::{
    routing::{delete, get, post},
    Router,
};
use state::AppState;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_app(state: AppState) -> Router {
    // CORS: restrict to configured origins when provided, otherwise allow any
    // (useful for local development with the unpacked extension).
    let cors = if state.config.cors_allowed_origins.is_empty() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins = state
            .config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect::<Vec<_>>();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    Router::new()
        .route("/", get(|| async { "ORBIT Backend is running" }))
        .route("/health", get(|| async { "ok" }))
        // Auth
        .route("/api/v1/auth/register", post(auth::handlers::register))
        .route("/api/v1/auth/login", post(auth::handlers::login))
        .route("/api/v1/auth/refresh", post(auth::handlers::refresh))
        // Devices (bearer-authenticated inside handlers)
        .route(
            "/api/v1/devices/register",
            post(device::handlers::register_device),
        )
        .route("/api/v1/devices", get(device::handlers::list_devices))
        .route(
            "/api/v1/devices/{device_id}",
            get(device::handlers::get_device),
        )
        .route(
            "/api/v1/devices/heartbeat",
            post(device::handlers::heartbeat),
        )
        // Contacts
        .route("/api/v1/contacts/add", post(contacts::handlers::add_contact))
        .route("/api/v1/contacts", get(contacts::handlers::list_contacts))
        .route(
            "/api/v1/contacts/{contact_id}",
            delete(contacts::handlers::remove_contact),
        )
        .route("/api/v1/contacts/search", get(contacts::handlers::search_users))
        // Sessions
        .route(
            "/api/v1/sessions/create",
            post(session::handlers::create_session),
        )
        .route(
            "/api/v1/sessions/{session_id}/accept",
            post(session::handlers::accept_session),
        )
        .route(
            "/api/v1/sessions/{session_id}",
            delete(session::handlers::end_session),
        )
        .route(
            "/api/v1/sessions/active",
            get(session::handlers::get_active_session),
        )
        .route(
            "/api/v1/sessions/pending",
            get(session::handlers::get_pending_sessions),
        )
        // WebRTC signaling (session token itself authorizes the room)
        .route("/ws/{session_id}", get(websocket::handler::ws_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
