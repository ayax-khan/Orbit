use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::auth::handlers::user_id_from_bearer;
use crate::auth::models::{
    DeviceSummary, RegisterDeviceRequest, RegisterDeviceResponse,
};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers.get("Authorization").and_then(|h| h.to_str().ok())
}

/// POST /api/v1/devices/register
///
/// Registers a desktop agent as a device owned by the authenticated user and
/// returns a unique device token used by the agent to authenticate later.
pub async fn register_device(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterDeviceRequest>,
) -> AppResult<Json<RegisterDeviceResponse>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    if payload.device_name.trim().is_empty() {
        return Err(AppError::BadRequest("device_name is required".into()));
    }

    let device_token = Uuid::new_v4().to_string();
    let device_type = payload.device_type.unwrap_or_else(|| "windows".into());
    let agent_version = payload.agent_version.unwrap_or_else(|| "1.0.0".into());

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO devices \
         (user_id, device_name, device_type, os_version, agent_version, device_token, public_key) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(user_id)
    .bind(payload.device_name.trim())
    .bind(device_type)
    .bind(payload.os_version)
    .bind(agent_version)
    .bind(&device_token)
    .bind(payload.public_key)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(RegisterDeviceResponse {
        device_id: row.0,
        device_token,
    }))
}

/// GET /api/v1/devices
pub async fn list_devices(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<DeviceSummary>>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    let rows: Vec<crate::auth::models::Device> = sqlx::query_as(
        "SELECT id, device_name, is_online, last_seen FROM devices \
         WHERE user_id = $1 ORDER BY device_name",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let devices = rows
        .into_iter()
        .map(|d| DeviceSummary {
            id: d.id,
            name: d.device_name,
            status: if d.is_online { "online".into() } else { "offline".into() },
            last_seen: d.last_seen,
        })
        .collect();

    Ok(Json(devices))
}

/// GET /api/v1/devices/{device_id}
pub async fn get_device(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(device_id): Path<Uuid>,
) -> AppResult<Json<DeviceSummary>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    let device: Option<crate::auth::models::Device> = sqlx::query_as(
        "SELECT id, device_name, is_online, last_seen FROM devices \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(device_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let device = device.ok_or(AppError::NotFound)?;

    Ok(Json(DeviceSummary {
        id: device.id,
        name: device.device_name,
        status: if device.is_online { "online".into() } else { "offline".into() },
        last_seen: device.last_seen,
    }))
}
