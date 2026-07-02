use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row for the `users` table.
#[allow(dead_code)] // email/full_name are read from the DB and used for future profile endpoints
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    // Accepted for session binding / device fingerprinting (spec), reserved for v1.1.
    #[serde(default)]
    #[allow(dead_code)]
    pub device_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
}

/// Database row for the `devices` table (subset used by the API).
#[derive(Debug, Clone, FromRow)]
pub struct Device {
    pub id: Uuid,
    pub device_name: String,
    pub is_online: bool,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_name: String,
    pub os_version: String,
    pub public_key: String,
    #[serde(default)]
    pub device_type: Option<String>,
    #[serde(default)]
    pub agent_version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterDeviceResponse {
    pub device_id: Uuid,
    pub device_token: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceSummary {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    #[serde(default)]
    pub host_device_id: Option<Uuid>,
    #[serde(default)]
    pub host_user_id: Option<Uuid>,
    #[serde(default)]
    pub client_device_id: Option<Uuid>, // For device-to-device sessions
    pub session_type: String, // 'device' or 'user'
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub session_type: String,
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<String>,
}
