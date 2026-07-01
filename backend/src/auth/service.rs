use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).expect("Failed to parse hash");
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn create_jwt(user_id: &str) -> String {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 86400; // 24 hours

    let claims = Claims {
        sub: user_id.to_owned(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Failed to create JWT")
}

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
