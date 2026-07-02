use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Contact {
    pub id: Uuid,
    pub contact_user_id: Uuid,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddContactRequest {
    pub contact_user_id: Uuid,
    pub display_name: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ContactResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserSearchResult {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
}
