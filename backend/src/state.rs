use crate::config::Config;
use redis::Client;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared application state injected into every handler via `State`.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Client,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: PgPool, redis: Client, config: Config) -> Self {
        Self {
            db,
            redis,
            config: Arc::new(config),
        }
    }
}
