use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};

pub async fn create_session(
    pool: &PgPool,
    host_device_id: Uuid,
    client_user_id: Uuid,
) -> Result<String, sqlx::Error> {
    let session_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(12);

    sqlx::query!(
        "INSERT INTO sessions (host_device_id, client_user_id, session_token, status, expires_at) VALUES ($1, $2, $3, $4, $5)",
        host_device_id,
        client_user_id,
        session_token,
        "active",
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(session_token)
}
