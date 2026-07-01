use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;
use crate::auth::service::Claims;

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    match auth_header {
        Some(token) => {
            let secret = env::var("JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_ref()),
                &Validation::default(),
            ).map_err(|_| StatusCode::UNAUTHORIZED)?;
            Ok(next.run(req).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
