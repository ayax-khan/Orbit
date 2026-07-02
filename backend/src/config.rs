use std::env;

/// Runtime configuration loaded once at startup from environment variables.
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    // Reserved for the Redis-backed session cache & rate limiting (spec Phase 4).
    #[allow(dead_code)]
    pub redis_url: String,
    pub jwt_secret: String,
    pub bind_address: String,
    pub cors_allowed_origins: Vec<String>,
    pub stun_server: String,
    pub turn_server: String,
    pub access_token_ttl_secs: i64,
    pub session_ttl_hours: i64,
}

impl Config {
    /// Load configuration from the environment (and `.env` if present).
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Config {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into()),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            cors_allowed_origins,
            stun_server: env::var("STUN_SERVER")
                .unwrap_or_else(|_| "stun:stun.l.google.com:19302".into()),
            turn_server: env::var("TURN_SERVER").unwrap_or_default(),
            access_token_ttl_secs: env::var("ACCESS_TOKEN_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(86_400), // 24h per spec
            session_ttl_hours: env::var("SESSION_TTL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(12), // 12h max per spec
        }
    }
}
