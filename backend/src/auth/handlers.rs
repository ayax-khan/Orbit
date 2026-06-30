use axum::{extract::State, Json};
use std::sync::Arc;
use crate::AppState;
use super::models::{AuthResponse, LoginRequest, RegisterRequest};
use super::service::{create_jwt, hash_password};

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Json<AuthResponse> {
    let password_hash = hash_password(&payload.password);
    
    // Database interaction would go here using state.db
    // For now returning dummy token
    let user_id = "dummy_user_id";
    let token = create_jwt(user_id);
    
    Json(AuthResponse {
        access_token: token.clone(),
        refresh_token: token,
    })
}

pub async fn login(
    Json(_payload): Json<LoginRequest>,
) -> Json<AuthResponse> {
    let token = create_jwt("dummy_user_id");
    
    Json(AuthResponse {
        access_token: token.clone(),
        refresh_token: token,
    })
}
