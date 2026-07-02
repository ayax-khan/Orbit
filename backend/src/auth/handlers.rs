use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use uuid::Uuid;

use super::models::{
    AccessTokenResponse, AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, User,
};
use super::service::{create_jwt, hash_password, verify_jwt, verify_password};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::utils::rate_limit::check_rate_limit;

/// Very small email sanity check. Full RFC validation is out of scope for v1.
fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    email.len() >= 3 && email.contains('@') && !email.starts_with('@') && !email.ends_with('@')
}

/// Issue an access token + persist a refresh token for a user.
async fn issue_tokens(state: &AppState, user_id: Uuid) -> AppResult<AuthResponse> {
    let access_token = create_jwt(
        &user_id.to_string(),
        &state.config.jwt_secret,
        state.config.access_token_ttl_secs,
    )?;

    let refresh_token = Uuid::new_v4().to_string();
    let refresh_hash = hash_password(&refresh_token)?;
    let expires_at = Utc::now() + Duration::days(30);

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, device_fingerprint, expires_at) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(&refresh_hash)
    .bind("default")
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok(AuthResponse {
        user_id,
        access_token,
        refresh_token,
    })
}

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    let email = payload.email.trim().to_lowercase();

    // Rate limit: 5 attempts per 5 minutes per email
    if !check_rate_limit(&state, &format!("auth_limit:{}", email), 5, 300).await? {
        return Err(AppError::BadRequest(
            "too many attempts, try again later".into(),
        ));
    }

    if !is_valid_email(&email) {
        return Err(AppError::BadRequest("invalid email address".into()));
    }
    if payload.password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    if payload.full_name.trim().is_empty() {
        return Err(AppError::BadRequest("full name is required".into()));
    }

    let existing: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::Conflict("email already registered".into()));
    }

    let password_hash = hash_password(&payload.password)?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, full_name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(payload.full_name.trim())
    .fetch_one(&state.db)
    .await?;

    Ok(Json(issue_tokens(&state, row.0).await?))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let email = payload.email.trim().to_lowercase();

    // Rate limit: 5 attempts per 5 minutes per email
    if !check_rate_limit(&state, &format!("auth_limit:{}", email), 5, 300).await? {
        return Err(AppError::BadRequest(
            "too many attempts, try again later".into(),
        ));
    }

    let user: Option<User> = sqlx::query_as(
        "SELECT id, email, password_hash, full_name FROM users WHERE email = $1 AND is_active = TRUE",
    )
    .bind(&email)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or(AppError::InvalidCredentials)?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::InvalidCredentials);
    }

    sqlx::query("UPDATE users SET last_login = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await?;

    Ok(Json(issue_tokens(&state, user.id).await?))
}

/// POST /api/v1/auth/refresh
///
/// Refresh tokens are stored hashed, so we look up all live tokens for
/// verification. For v1 scale this is acceptable; a token-id scheme can be
/// introduced later for O(1) lookups.
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> AppResult<Json<AccessTokenResponse>> {
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT user_id, token_hash FROM refresh_tokens WHERE expires_at > CURRENT_TIMESTAMP",
    )
    .fetch_all(&state.db)
    .await?;

    let user_id = rows
        .into_iter()
        .find(|(_, hash)| verify_password(&payload.refresh_token, hash))
        .map(|(uid, _)| uid)
        .ok_or(AppError::Unauthorized)?;

    let access_token = create_jwt(
        &user_id.to_string(),
        &state.config.jwt_secret,
        state.config.access_token_ttl_secs,
    )?;

    Ok(Json(AccessTokenResponse { access_token }))
}

/// Extract and validate the authenticated user id from a bearer token.
pub fn user_id_from_bearer(state: &AppState, header: Option<&str>) -> AppResult<Uuid> {
    let token = header
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let claims = verify_jwt(token, &state.config.jwt_secret)?;
    Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)
}
