use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub hashed_password: String,
    pub role: String, // "admin" or "staff"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String, // "todo", "in_progress", "done"
    pub priority: String, // "low", "medium", "high"
    pub created_by_id: Uuid,
    pub assigned_to_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoginChallenge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String, // hashed
    pub code_plain: Option<String>, // only in memory for logging
    pub attempts: i32,
    pub expires_at: DateTime<Utc>,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailLog {
    pub id: Uuid,
    pub to_email: String,
    pub subject: String,
    pub body: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub login_challenge_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Verify2FARequest {
    pub login_challenge_id: Uuid,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Verify2FAResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignTasksRequest {
    pub task_ids: Vec<Uuid>,
    pub assign_to_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<String>, // email
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewMyTasksResponse {
    pub user: UserResponse,
    pub tasks: Vec<TaskResponse>,
    pub summary: TaskSummary,
    pub cache: CacheMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSummary {
    pub total_assigned_tasks: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub hit: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub role: String,
    pub exp: i64,
}
