pub mod auth;
pub mod cache;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{routing::{get, post}, Router};
use redis::aio::ConnectionManager;
use sqlx::postgres::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub struct AppState {
    pub pool: PgPool,
    pub cache: Arc<cache::Cache>,
}

pub async fn create_app_state(database_url: &str, redis_url: &str) -> anyhow::Result<Arc<AppState>> {
    let pool = PgPool::connect(database_url).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    let redis_client = redis::Client::open(redis_url)?;
    let redis_manager = ConnectionManager::new(redis_client).await?;
    let cache = Arc::new(cache::Cache::new(redis_manager));

    Ok(Arc::new(AppState { pool, cache }))
}

pub fn build_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/seed/users", post(handlers::seed_users))
        .route("/auth/login", post(handlers::auth_login))
        .route("/auth/verify-2fa", post(handlers::verify_2fa))
        .route("/dev/email-logs/latest", get(handlers::get_latest_email_log))
        .route(
            "/tasks",
            post(handlers::create_task)
                .get(handlers::list_tasks),
        )
        .route("/tasks/assign", post(handlers::assign_tasks))
        .route("/tasks/view-my-tasks", get(handlers::view_my_tasks))
        .route("/health", get(handlers::health_check))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
