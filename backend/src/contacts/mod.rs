pub mod handlers;
pub mod models;

use axum::routing::{delete, get, post};
use axum::Router;

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/add", post(handlers::add_contact))
        .route("/", get(handlers::list_contacts))
        .route("/{contact_id}", delete(handlers::remove_contact))
        .route("/search", get(handlers::search_users))
}
