pub mod handlers;

use axum::routing::{delete, get, post};
use axum::Router;

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/create", post(handlers::create_session))
        .route("/{session_id}/accept", post(handlers::accept_session))
        .route("/{session_id}", delete(handlers::end_session))
        .route("/active", get(handlers::get_active_session))
}
