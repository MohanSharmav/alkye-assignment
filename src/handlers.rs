use crate::cache::cache_key_for_user_tasks;
use crate::db;
use crate::error::{ApiError, ApiResult};
use crate::middleware::AuthUser;
use crate::models::*;
use crate::auth::{self, hash_password, verify_password, generate_jwt, hash_verification_code};
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use chrono::Utc;

use crate::AppState;

// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

// Seed users endpoint
pub async fn seed_users(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    // Create Admin user
    let admin_password = hash_password("admin123")?;
    let admin = db::create_user(
        &state.pool,
        "Admin User",
        "admin@example.com",
        &admin_password,
        "admin",
    )
    .await?;

    // Create James Bond user
    let james_password = hash_password("james123")?;
    let james = db::create_user(
        &state.pool,
        "James Bond",
        "jamesbond@example.com",
        &james_password,
        "staff",
    )
    .await?;

    Ok(Json(json!({
        "message": "Users seeded successfully",
        "admin": {
            "id": admin.id,
            "email": admin.email,
            "role": admin.role,
        },
        "james_bond": {
            "id": james.id,
            "email": james.email,
            "role": james.role,
        }
    })))
}

// Auth login endpoint
pub async fn auth_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    // Find user by email
    let user = db::get_user_by_email(&state.pool, &payload.email)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

    // Verify password
    let password_valid = verify_password(&payload.password, &user.hashed_password)?;
    if !password_valid {
        return Err(ApiError::InvalidCredentials);
    }

    // Generate verification code
    let code = auth::generate_verification_code();
    let hashed_code = hash_verification_code(&code);
    let expires_at = auth::get_verification_code_expiry();

    // Create login challenge
    let challenge = db::create_login_challenge(&state.pool, user.id, &hashed_code, expires_at).await?;

    // Log email with code
    db::create_email_log(
        &state.pool,
        &user.email,
        "Your 2FA Verification Code",
        &format!("Your verification code is: {}", code),
        Some(&code),
    )
    .await?;

    tracing::info!("Login challenge created for {}: code={}", user.email, code);

    Ok(Json(LoginResponse {
        login_challenge_id: challenge.id,
        message: "Verification code sent to email".to_string(),
    }))
}

// Verify 2FA endpoint
pub async fn verify_2fa(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Verify2FARequest>,
) -> ApiResult<Json<Verify2FAResponse>> {
    // Get challenge
    let challenge = db::get_login_challenge(&state.pool, payload.login_challenge_id)
        .await?
        .ok_or(ApiError::Invalid2FACode)?;

    // Check if expired
    if Utc::now() > challenge.expires_at {
        return Err(ApiError::Expired2FACode);
    }

    // Check if already verified
    if challenge.verified {
        return Err(ApiError::Used2FACode);
    }

    // Check attempts
    if challenge.attempts >= 3 {
        return Err(ApiError::Invalid2FACode);
    }

    // Verify code
    let hashed_input = hash_verification_code(&payload.code);
    if hashed_input != challenge.code {
        db::increment_challenge_attempts(&state.pool, challenge.id).await?;
        return Err(ApiError::Invalid2FACode);
    }

    // Mark as verified
    db::verify_login_challenge(&state.pool, challenge.id).await?;

    // Get user
    let user = db::get_user_by_id(&state.pool, challenge.user_id)
        .await?
        .ok_or(ApiError::UserNotFound)?;

    // Generate JWT
    let access_token = generate_jwt(user.id, user.email.clone(), user.role.clone())?;

    Ok(Json(Verify2FAResponse {
        access_token,
        token_type: "Bearer".to_string(),
        user: UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
        },
    }))
}

// Get latest email log (dev endpoint)
pub async fn get_latest_email_log(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Option<EmailLog>>> {
    let log = db::get_latest_email_log(&state.pool).await?;
    Ok(Json(log))
}

// Create task (admin only)
pub async fn create_task(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<CreateTaskRequest>,
) -> ApiResult<(StatusCode, Json<TaskResponse>)> {
    // Check admin role
    auth_user.require_admin()?;

    // Create task
    let task = db::create_task(
        &state.pool,
        &payload.title,
        payload.description.as_deref(),
        &payload.priority,
        auth_user.user_id,
    )
    .await?;

    // Get assigned user info if applicable
    let assigned_to = if let Some(assigned_id) = task.assigned_to_id {
        let user = db::get_user_by_id(&state.pool, assigned_id).await?;
        user.map(|u| u.email)
    } else {
        None
    };

    Ok((
        StatusCode::CREATED,
        Json(TaskResponse {
            id: task.id,
            title: task.title,
            description: task.description,
            status: task.status,
            priority: task.priority,
            assigned_to,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }),
    ))
}

// List all tasks
pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> ApiResult<Json<Vec<TaskResponse>>> {
    auth_user.require_admin()?;

    let tasks = db::list_all_tasks(&state.pool).await?;

    let response = futures::future::join_all(
        tasks.into_iter().map(|task| {
            let state = state.clone();
            async move {
                let assigned_to = if let Some(assigned_id) = task.assigned_to_id {
                    db::get_user_by_id(&state.pool, assigned_id)
                        .await
                        .ok()
                        .flatten()
                        .map(|u| u.email)
                } else {
                    None
                };

                TaskResponse {
                    id: task.id,
                    title: task.title,
                    description: task.description,
                    status: task.status,
                    priority: task.priority,
                    assigned_to,
                    created_at: task.created_at,
                    updated_at: task.updated_at,
                }
            }
        })
    )
    .await;

    Ok(Json(response))
}

// Assign tasks (admin only)
pub async fn assign_tasks(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<AssignTasksRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check admin role
    auth_user.require_admin()?;

    // Get the target user
    let target_user = db::get_user_by_email(&state.pool, &payload.assign_to_email)
        .await?
        .ok_or(ApiError::UserNotFound)?;

    // Assign each task
    for task_id in &payload.task_ids {
        db::assign_task(&state.pool, *task_id, target_user.id).await?;
        
        // Invalidate cache for the assigned user
        state.cache.delete(&cache_key_for_user_tasks(target_user.id)).await?;
    }

    Ok(Json(json!({
        "message": format!("Assigned {} tasks to {}", payload.task_ids.len(), payload.assign_to_email),
        "assigned_count": payload.task_ids.len(),
    })))
}

// View my tasks (with caching)
pub async fn view_my_tasks(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> ApiResult<Json<ViewMyTasksResponse>> {
    let cache_key = cache_key_for_user_tasks(auth_user.user_id);

    // Try to get from cache first
    if let Ok(Some(cached)) = state.cache.get::<ViewMyTasksResponse>(&cache_key).await {
        return Ok(Json(ViewMyTasksResponse {
            cache: CacheMetadata { hit: true },
            ..cached
        }));
    }

    // Get user
    let user = db::get_user_by_id(&state.pool, auth_user.user_id)
        .await?
        .ok_or(ApiError::UserNotFound)?;

    // Get tasks for user
    let tasks = db::get_tasks_for_user(&state.pool, auth_user.user_id).await?;

    // Build response
    let task_responses = futures::future::join_all(
        tasks.into_iter().map(|task| {
            let state = state.clone();
            async move {
                let assigned_to = if let Some(assigned_id) = task.assigned_to_id {
                    db::get_user_by_id(&state.pool, assigned_id)
                        .await
                        .ok()
                        .flatten()
                        .map(|u| u.email)
                } else {
                    None
                };

                TaskResponse {
                    id: task.id,
                    title: task.title,
                    description: task.description,
                    status: task.status,
                    priority: task.priority,
                    assigned_to,
                    created_at: task.created_at,
                    updated_at: task.updated_at,
                }
            }
        })
    )
    .await;

    let response = ViewMyTasksResponse {
        user: UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
        },
        summary: TaskSummary {
            total_assigned_tasks: task_responses.len(),
        },
        tasks: task_responses,
        cache: CacheMetadata { hit: false },
    };

    // Cache the response for 5 minutes
    let _ = state.cache.set(&cache_key, &response, 300).await;

    Ok(Json(response))
}
