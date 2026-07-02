mod auth;
mod config;
mod db;
mod device;
mod error;
mod session;
mod state;
mod utils;
mod websocket;

use orbit_backend::config::Config;
use orbit_backend::create_app;
use orbit_backend::state::AppState;

#[tokio::main]
async fn main() {
    // Structured logging (RUST_LOG controls verbosity).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env();
    let bind_address = config.bind_address.clone();

    let db_pool = db::connection::establish_connection(&config.database_url).await;
    let redis_client =
        redis::Client::open(config.redis_url.clone()).expect("failed to create redis client");

    // Apply pending migrations automatically on startup.
    if let Err(e) = sqlx::migrate!("./migrations").run(&db_pool).await {
        tracing::error!("failed to run migrations: {e}");
        std::process::exit(1);
    }

    let state = AppState::new(db_pool, redis_client, config);
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {bind_address}: {e}"));
    tracing::info!("ORBIT backend listening on http://{bind_address}");
    axum::serve(listener, app).await.unwrap();
}
