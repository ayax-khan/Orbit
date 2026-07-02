use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::auth::handlers::user_id_from_bearer;
use crate::contacts::models::{AddContactRequest, Contact, ContactResponse, UserSearchResult};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers.get("Authorization").and_then(|h| h.to_str().ok())
}

/// POST /api/v1/contacts/add
///
/// Adds a user to the authenticated user's contacts list.
pub async fn add_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AddContactRequest>,
) -> AppResult<Json<ContactResponse>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    // Prevent adding self
    if user_id == payload.contact_user_id {
        return Err(AppError::BadRequest("Cannot add yourself as a contact".to_string()));
    }

    // Check if contact exists
    let contact_user: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT id, email, full_name FROM users WHERE id = $1 AND is_active = TRUE",
    )
    .bind(payload.contact_user_id)
    .fetch_optional(&state.db)
    .await?;

    let (contact_id, email, full_name) = contact_user.ok_or(AppError::NotFound)?;

    // Add to contacts (ignore if already exists)
    sqlx::query(
        "INSERT INTO contacts (user_id, contact_user_id, display_name) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, contact_user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(payload.contact_user_id)
    .bind(&payload.display_name)
    .execute(&state.db)
    .await?;

    let contact: ContactResponse = sqlx::query_as(
        "SELECT c.id, u.id as user_id, u.email, u.full_name, c.display_name, c.created_at \
         FROM contacts c \
         JOIN users u ON c.contact_user_id = u.id \
         WHERE c.user_id = $1 AND c.contact_user_id = $2",
    )
    .bind(user_id)
    .bind(payload.contact_user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(contact))
}

/// GET /api/v1/contacts
///
/// Lists all contacts for the authenticated user.
pub async fn list_contacts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<ContactResponse>>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    let contacts: Vec<ContactResponse> = sqlx::query_as(
        "SELECT c.id, u.id as user_id, u.email, u.full_name, c.display_name, c.created_at \
         FROM contacts c \
         JOIN users u ON c.contact_user_id = u.id \
         WHERE c.user_id = $1 \
         ORDER BY c.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(contacts))
}

/// DELETE /api/v1/contacts/{contact_id}
///
/// Removes a contact from the authenticated user's contacts list.
pub async fn remove_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(contact_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    let updated = sqlx::query("DELETE FROM contacts WHERE id = $1 AND user_id = $2")
        .bind(contact_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "status": "removed" })))
}

/// GET /api/v1/contacts/search?q=query
///
/// Searches for users by email or full name.
pub async fn search_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Vec<UserSearchResult>>> {
    let user_id = user_id_from_bearer(&state, bearer(&headers))?;

    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let search_pattern = format!("%{}%", query);

    let users: Vec<UserSearchResult> = sqlx::query_as(
        "SELECT id, email, full_name FROM users \
         WHERE is_active = TRUE \
         AND id != $1 \
         AND (email ILIKE $2 OR full_name ILIKE $2) \
         LIMIT 20",
    )
    .bind(user_id)
    .bind(&search_pattern)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(users))
}
