use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use orbit_backend::create_app;
use orbit_backend::state::AppState;
use orbit_backend::config::Config;
use orbit_backend::db::connection::establish_connection;
use serde_json::json;

async fn create_test_state() -> AppState {
    let config = Config::from_env();
    let db_pool = establish_connection(&config.database_url).await;
    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("failed to create redis client");
    AppState::new(db_pool, redis_client, config)
}

#[tokio::test]
async fn test_device_lifecycle() {
    let state = create_test_state().await;
    let app = create_app(state);

    // 1. Create a user and get a token
    let email = format!("dev_test_{}@example.com", uuid::Uuid::new_v4());
    
    // Register
    app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "email": email,
                "password": "password123",
                "full_name": "Device Test User"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    // Login
    let login_response = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "email": email,
                "password": "password123"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    let login_body: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(login_response.into_body(), usize::MAX).await.unwrap()).unwrap();
    let token = login_body["access_token"].as_str().unwrap();

    // 2. Register a device
    let reg_device_response = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/devices/register")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(json!({
                "device_name": "Test Desktop Agent",
                "device_type": "windows",
                "os_version": "11.0",
                "public_key": "dummy_key"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    // Print body if it failed for debugging
    if reg_device_response.status() != StatusCode::OK {
        let body = axum::body::to_bytes(reg_device_response.into_body(), usize::MAX).await.unwrap();
        panic!("Register device failed: {}", String::from_utf8_lossy(&body));
    }
    assert_eq!(reg_device_response.status(), StatusCode::OK);

    // 3. List devices
    let list_response = app.clone().oneshot(
        Request::builder()
            .method("GET")
            .uri("/api/v1/devices")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
}
