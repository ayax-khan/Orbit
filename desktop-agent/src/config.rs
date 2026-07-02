//! Agent runtime configuration.
//!
//! Values are loaded from environment variables with sensible defaults so the
//! agent runs out of the box for development while remaining configurable in
//! production (per the spec's config-management design).

use std::env;

#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Backend authority, e.g. `127.0.0.1:8080`.
    pub backend_host: String,
    /// Session id (room) to join for signaling.
    pub session_id: String,
    /// ICE server URLs (STUN/TURN). Empty falls back to a public STUN server.
    pub ice_servers: Vec<String>,
    /// Preferred capture dimensions (used by the portable fallback capturer).
    pub capture_width: u32,
    pub capture_height: u32,
    /// Adaptive FPS bounds (spec: 30 min, 60 target).
    pub min_fps: u32,
    pub max_fps: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            backend_host: "127.0.0.1:8080".into(),
            session_id: String::new(),
            ice_servers: Vec::new(),
            capture_width: 1920,
            capture_height: 1080,
            min_fps: 30,
            max_fps: 60,
        }
    }
}

impl AgentConfig {
    /// Load configuration from the environment, falling back to defaults.
    pub fn from_env() -> Self {
        let defaults = Self::default();

        let ice_servers = env::var("ORBIT_ICE_SERVERS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            backend_host: env::var("ORBIT_BACKEND_HOST").unwrap_or(defaults.backend_host),
            session_id: env::var("ORBIT_SESSION_ID").unwrap_or(defaults.session_id),
            ice_servers,
            capture_width: parse_env("ORBIT_CAPTURE_WIDTH", defaults.capture_width),
            capture_height: parse_env("ORBIT_CAPTURE_HEIGHT", defaults.capture_height),
            min_fps: parse_env("ORBIT_MIN_FPS", defaults.min_fps),
            max_fps: parse_env("ORBIT_MAX_FPS", defaults.max_fps),
        }
    }
}

fn parse_env(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
