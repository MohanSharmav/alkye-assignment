use crate::error::ApiResult;
use crate::models::{User, Task, LoginChallenge, EmailLog};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// User queries
pub async fn create_user(
    pool: &PgPool,
    full_name: &str,
    email: &str,
    hashed_password: &str,
    role: &str,
) -> ApiResult<User> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, full_name, email, hashed_password, role, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *"
    )
    .bind(Uuid::new_v4())
    .bind(full_name)
    .bind(email)
    .bind(hashed_password)
    .bind(role)
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> ApiResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

// Task queries
pub async fn create_task(
    pool: &PgPool,
    title: &str,
    description: Option<&str>,
    priority: &str,
    created_by_id: Uuid,
) -> ApiResult<Task> {
    let task = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (id, title, description, status, priority, created_by_id, assigned_to_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING *"
    )
    .bind(Uuid::new_v4())
    .bind(title)
    .bind(description)
    .bind("todo")
    .bind(priority)
    .bind(created_by_id)
    .bind::<Option<Uuid>>(None)
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    Ok(task)
}

pub async fn assign_task(
    pool: &PgPool,
    task_id: Uuid,
    assign_to_id: Uuid,
) -> ApiResult<Task> {
    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET assigned_to_id = $1, updated_at = $2 WHERE id = $3 RETURNING *"
    )
    .bind(assign_to_id)
    .bind(Utc::now())
    .bind(task_id)
    .fetch_one(pool)
    .await?;

    Ok(task)
}

pub async fn get_tasks_for_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<Task>> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE assigned_to_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

pub async fn get_task_by_id(pool: &PgPool, task_id: Uuid) -> ApiResult<Option<Task>> {
    let task = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE id = $1"
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

pub async fn list_all_tasks(pool: &PgPool) -> ApiResult<Vec<Task>> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

// LoginChallenge queries
pub async fn create_login_challenge(
    pool: &PgPool,
    user_id: Uuid,
    hashed_code: &str,
    expires_at: DateTime<Utc>,
) -> ApiResult<LoginChallenge> {
    let challenge = sqlx::query_as::<_, LoginChallenge>(
        "INSERT INTO login_challenges (id, user_id, code, attempts, expires_at, verified, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *"
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(hashed_code)
    .bind(0)
    .bind(expires_at)
    .bind(false)
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    Ok(challenge)
}

pub async fn get_login_challenge(pool: &PgPool, challenge_id: Uuid) -> ApiResult<Option<LoginChallenge>> {
    let challenge = sqlx::query_as::<_, LoginChallenge>(
        "SELECT * FROM login_challenges WHERE id = $1"
    )
    .bind(challenge_id)
    .fetch_optional(pool)
    .await?;

    Ok(challenge)
}

pub async fn verify_login_challenge(
    pool: &PgPool,
    challenge_id: Uuid,
) -> ApiResult<LoginChallenge> {
    let challenge = sqlx::query_as::<_, LoginChallenge>(
        "UPDATE login_challenges SET verified = true, updated_at = $1 WHERE id = $2 RETURNING *"
    )
    .bind(Utc::now())
    .bind(challenge_id)
    .fetch_one(pool)
    .await?;

    Ok(challenge)
}

pub async fn increment_challenge_attempts(
    pool: &PgPool,
    challenge_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        "UPDATE login_challenges SET attempts = attempts + 1 WHERE id = $1"
    )
    .bind(challenge_id)
    .execute(pool)
    .await?;

    Ok(())
}

// EmailLog queries
pub async fn create_email_log(
    pool: &PgPool,
    to_email: &str,
    subject: &str,
    body: &str,
    code: Option<&str>,
) -> ApiResult<EmailLog> {
    let log = sqlx::query_as::<_, EmailLog>(
        "INSERT INTO email_logs (id, to_email, subject, body, code, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
    .bind(Uuid::new_v4())
    .bind(to_email)
    .bind(subject)
    .bind(body)
    .bind(code)
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    Ok(log)
}

pub async fn get_latest_email_log(pool: &PgPool) -> ApiResult<Option<EmailLog>> {
    let log = sqlx::query_as::<_, EmailLog>(
        "SELECT * FROM email_logs ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    Ok(log)
}
