use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::auth::handlers::user_id_from_bearer;
use crate::auth::models::{CreateSessionRequest, CreateSessionResponse};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers.get("Authorization").and_then(|h| h.to_str().ok())
}

/// POST /api/v1/sessions/create
///
/// Creates a session for the authenticated client to connect to either:
/// - A host device (device session)
/// - Another user (user-to-user session for extension-to-extension screen sharing)
pub async fn create_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateSessionRequest>,
) -> AppResult<Json<CreateSessionResponse>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;
    let session_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(state.config.session_ttl_hours);

    match payload.session_type.as_str() {
        "device" => {
            // Device session: can be either:
            // 1. Extension connects to own device (host_device_id only)
            // 2. Device connects to another device (host_device_id + client_device_id)
            
            let host_device_id = payload.host_device_id.ok_or(AppError::BadRequest(
                "host_device_id required for device sessions".to_string(),
            ))?;

            if let Some(client_device_id) = payload.client_device_id {
                // Device-to-device session
                // Verify both devices exist and allow remote access
                let host_device: Option<(Uuid, bool, Uuid)> = sqlx::query_as(
                    "SELECT id, allow_remote_access, user_id FROM devices WHERE id = $1",
                )
                .bind(host_device_id)
                .fetch_optional(&state.db)
                .await?;

                let (host_id, allow_remote, host_user) = host_device.ok_or(AppError::NotFound)?;
                if !allow_remote {
                    return Err(AppError::BadRequest("Host device does not allow remote access".to_string()));
                }

                let client_device: Option<(Uuid, Uuid)> = sqlx::query_as(
                    "SELECT id, user_id FROM devices WHERE id = $1",
                )
                .bind(client_device_id)
                .fetch_optional(&state.db)
                .await?;

                let (client_id, client_user) = client_device.ok_or(AppError::NotFound)?;

                // Verify the requesting user owns the client device
                if client_user != user_id {
                    return Err(AppError::Unauthorized);
                }

                sqlx::query(
                    "INSERT INTO sessions \
                     (host_device_id, client_device_id, client_user_id, session_token, status, expires_at, session_type) \
                     VALUES ($1, $2, $3, $4, 'pending', $5, 'device')",
                )
                .bind(host_id)
                .bind(client_id)
                .bind(user_id)
                .bind(&session_token)
                .bind(expires_at)
                .execute(&state.db)
                .await?;
            } else {
                // Extension-to-device session (existing behavior)
                let device: Option<(Uuid, bool)> = sqlx::query_as(
                    "SELECT id, allow_remote_access FROM devices WHERE id = $1 AND user_id = $2",
                )
                .bind(host_device_id)
                .bind(user_id)
                .fetch_optional(&state.db)
                .await?;

                let (device_id, allow_remote) = device.ok_or(AppError::NotFound)?;
                if !allow_remote {
                    return Err(AppError::Unauthorized);
                }

                sqlx::query(
                    "INSERT INTO sessions \
                     (host_device_id, client_user_id, session_token, status, expires_at, session_type) \
                     VALUES ($1, $2, $3, 'active', $4, 'device')",
                )
                .bind(device_id)
                .bind(user_id)
                .bind(&session_token)
                .bind(expires_at)
                .execute(&state.db)
                    .await?;
            }
        }
        "user" => {
            // User session: client connects to another user (extension-to-extension)
            let host_user_id = payload.host_user_id.ok_or(AppError::BadRequest(
                "host_user_id required for user sessions".to_string(),
            ))?;

            // Verify the host user exists and is active
            let host_exists: Option<(bool,)> = sqlx::query_as(
                "SELECT is_active FROM users WHERE id = $1",
            )
            .bind(host_user_id)
            .fetch_optional(&state.db)
            .await?;

            let (is_active,) = host_exists.ok_or(AppError::NotFound)?;
            if !is_active {
                return Err(AppError::BadRequest("Host user is not active".to_string()));
            }

            sqlx::query(
                "INSERT INTO sessions \
                 (host_user_id, client_user_id, session_token, status, expires_at, session_type) \
                 VALUES ($1, $2, $3, 'pending', $4, 'user')",
            )
            .bind(host_user_id)
            .bind(user_id)
            .bind(&session_token)
            .bind(expires_at)
            .execute(&state.db)
            .await?;
        }
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid session_type: {}. Must be 'device' or 'user'",
                payload.session_type
            )));
        }
    }

    let turn_servers = if state.config.turn_server.is_empty() {
        vec![]
    } else {
        vec![state.config.turn_server.clone()]
    };

    Ok(Json(CreateSessionResponse {
        session_id: session_token,
        session_type: payload.session_type,
        stun_servers: vec![state.config.stun_server.clone()],
        turn_servers,
    }))
}

/// POST /api/v1/sessions/{session_id}/accept
pub async fn accept_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let updated = sqlx::query(
        "UPDATE sessions SET status = 'active' WHERE session_token = $1 AND status <> 'expired'",
    )
    .bind(&session_id)
    .execute(&state.db)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "status": "accepted" })))
}

/// DELETE /api/v1/sessions/{session_id}
pub async fn end_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    let updated = sqlx::query(
        "UPDATE sessions SET status = 'inactive', ended_at = CURRENT_TIMESTAMP \
         WHERE session_token = $1 AND client_user_id = $2",
    )
    .bind(&session_id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "status": "disconnected" })))
}

/// GET /api/v1/sessions/active
pub async fn get_active_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<CreateSessionResponse>> {
    let device_token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let session: Option<(String,)> = sqlx::query_as(
        "SELECT s.session_token FROM sessions s \
         JOIN devices d ON s.host_device_id = d.id \
         WHERE d.device_token = $1 AND s.status = 'active' \
         ORDER BY s.started_at DESC LIMIT 1",
    )
    .bind(device_token)
    .fetch_optional(&state.db)
    .await?;

    let (session_id,) = session.ok_or(AppError::NotFound)?;

    Ok(Json(CreateSessionResponse {
        session_id,
        session_type: "device".to_string(),
        stun_servers: vec![state.config.stun_server.clone()],
        turn_servers: if state.config.turn_server.is_empty() {
            vec![]
        } else {
            vec![state.config.turn_server.clone()]
        },
    }))
}

/// GET /api/v1/sessions/pending
///
/// Gets pending session requests:
/// - For users: pending user-to-user session requests (as host)
/// - For devices: pending device-to-device session requests (as host device)
pub async fn get_pending_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // Check if this is a device token or user token
    let token = bearer(&headers).ok_or(AppError::Unauthorized)?;
    
    // First try as device token
    let device_sessions: Option<Vec<(Uuid, String, String, DateTime<Utc>, String)>> = sqlx::query_as(
        "SELECT s.id, s.session_token, d.device_name, s.started_at, u.full_name \
         FROM sessions s \
         JOIN devices d ON s.client_device_id = d.id \
         JOIN users u ON d.user_id = u.id \
         WHERE s.host_device_id = (SELECT id FROM devices WHERE device_token = $1) \
         AND s.session_type = 'device' AND s.client_device_id IS NOT NULL AND s.status = 'pending' \
         ORDER BY s.started_at DESC",
    )
    .bind(token.strip_prefix("Bearer ").unwrap_or(token))
    .fetch_optional(&state.db)
    .await?
    .map(|sessions| vec![sessions]); // Convert single result to vec

    if let Some(sessions) = device_sessions {
        let result: Vec<serde_json::Value> = sessions
            .into_iter()
            .map(|(id, token, device_name, started_at, user_name)| {
                serde_json::json!({
                    "id": id,
                    "session_id": token,
                    "client_device_name": device_name,
                    "client_user_name": user_name,
                    "started_at": started_at,
                    "session_type": "device"
                })
            })
            .collect();
        return Ok(Json(result));
    }

    // Fall back to user token (user-to-user sessions)
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    let sessions: Vec<(Uuid, String, String, DateTime<Utc>, String)> = sqlx::query_as(
        "SELECT s.id, s.session_token, u.email, s.started_at, u.full_name \
         FROM sessions s \
         JOIN users u ON s.client_user_id = u.id \
         WHERE s.host_user_id = $1 AND s.session_type = 'user' AND s.status = 'pending' \
         ORDER BY s.started_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let result: Vec<serde_json::Value> = sessions
        .into_iter()
        .map(|(id, token, email, started_at, full_name)| {
            serde_json::json!({
                "id": id,
                "session_id": token,
                "client_email": email,
                "client_name": full_name,
                "started_at": started_at,
                "session_type": "user"
            })
        })
        .collect();

    Ok(Json(result))
}
