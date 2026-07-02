use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use chrono::{Duration, Utc};
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
/// Creates a session for the authenticated client to connect to a host device.
/// The device must be owned by the requesting user and permit remote access
/// (ownership/authorization checks per spec's Security Architecture).
pub async fn create_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateSessionRequest>,
) -> AppResult<Json<CreateSessionResponse>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    // Verify the device exists, is owned by the user, and allows remote access.
    let device: Option<(Uuid, bool)> = sqlx::query_as(
        "SELECT id, allow_remote_access FROM devices WHERE id = $1 AND user_id = $2",
    )
    .bind(payload.host_device_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let (device_id, allow_remote) = device.ok_or(AppError::NotFound)?;
    if !allow_remote {
        return Err(AppError::Unauthorized);
    }

    let session_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(state.config.session_ttl_hours);

    sqlx::query(
        "INSERT INTO sessions \
         (host_device_id, client_user_id, session_token, status, expires_at) \
         VALUES ($1, $2, $3, 'active', $4)",
    )
    .bind(device_id)
    .bind(user_id)
    .bind(&session_token)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    let turn_servers = if state.config.turn_server.is_empty() {
        vec![]
    } else {
        vec![state.config.turn_server.clone()]
    };

    Ok(Json(CreateSessionResponse {
        session_id: session_token,
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
