mod auth;
mod cache;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Create database pool
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    // Initialize Redis cache
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;
    let redis_manager = redis::aio::ConnectionManager::new(redis_client).await?;
    let cache = Arc::new(cache::Cache::new(redis_manager));

    // Build router
    let app = Router::new()
        // Seed endpoint
        .route("/seed/users", post(handlers::seed_users))
        // Auth endpoints
        .route("/auth/login", post(handlers::auth_login))
        .route("/auth/verify-2fa", post(handlers::verify_2fa))
        // Dev endpoints
        .route("/dev/email-logs/latest", get(handlers::get_latest_email_log))
        // Task endpoints
        .route(
            "/tasks",
            post(handlers::create_task)
                .get(handlers::list_tasks),
        )
        .route("/tasks/assign", post(handlers::assign_tasks))
        .route("/tasks/view-my-tasks", get(handlers::view_my_tasks))
        // Health check
        .route("/health", get(handlers::health_check))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(AppState { pool, cache }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server listening on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await?;

    Ok(())
}

pub struct AppState {
    pub pool: PgPool,
    pub cache: Arc<cache::Cache>,
}
