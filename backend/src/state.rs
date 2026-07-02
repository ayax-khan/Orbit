use crate::config::Config;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared application state injected into every handler via `State`.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config) -> Self {
        Self {
            db,
            config: Arc::new(config),
        }
    }
}
