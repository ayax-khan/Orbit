use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use orbit_backend::config::Config;
use orbit_backend::create_app;
use orbit_backend::db::connection::establish_connection;
use orbit_backend::state::AppState;
use serde_json::json;
use tower::ServiceExt;

// Helper to create an app state
async fn create_test_state() -> AppState {
    let config = Config::from_env();
    let db_pool = establish_connection(&config.database_url).await;
    let redis_client =
        redis::Client::open(config.redis_url.clone()).expect("failed to create redis client");
    AppState::new(db_pool, redis_client, config)
}

#[tokio::test]
async fn test_health_check() {
    let state = create_test_state().await;
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_register_and_login_flow() {
    let state = create_test_state().await;
    let app = create_app(state);

    let email = format!("test_{}@example.com", uuid::Uuid::new_v4());

    // 1. Register
    let reg_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "password": "password123",
                        "full_name": "Test User"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reg_response.status(), StatusCode::OK);

    // 2. Login
    let login_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "password": "password123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);
}
