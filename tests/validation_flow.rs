use axum::{extract::State, http::StatusCode, Json};
use dotenv::dotenv;
use serde_json::Value;
use sqlx::query;
use std::{env, sync::Arc};
use task_api::{auth, create_app_state, handlers, middleware::AuthUser, models::{AssignTasksRequest, CreateTaskRequest, LoginRequest, Verify2FARequest}};
use uuid::Uuid;

#[tokio::test]
async fn validation_flow() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let state = create_app_state(&database_url, &redis_url).await?;

    // Clean up stale test state so the workflow can run repeatedly.
    query("DELETE FROM tasks").execute(&state.pool).await?;
    query("DELETE FROM login_challenges").execute(&state.pool).await?;
    query("DELETE FROM email_logs").execute(&state.pool).await?;
    query("DELETE FROM users").execute(&state.pool).await?;

    // 1. Seed Admin and James Bond
    let seed_response = handlers::seed_users(State(state.clone())).await?;
    assert_eq!(seed_response.0["message"], "Users seeded successfully");

    // 2. Admin login starts 2FA
    let admin_login_response = handlers::auth_login(
        State(state.clone()),
        Json(LoginRequest {
            email: "admin@example.com".to_string(),
            password: "admin123".to_string(),
        }),
    )
    .await?;
    let admin_challenge_id = admin_login_response.login_challenge_id;

    // 3. Retrieve admin verification code from dev email log
    let email_log = handlers::get_latest_email_log(State(state.clone())).await?.0;
    let admin_code = email_log.code.expect("admin verification code missing");

    // 4. Verify admin 2FA and get JWT
    let admin_verify_response = handlers::verify_2fa(
        State(state.clone()),
        Json(Verify2FARequest {
            login_challenge_id: admin_challenge_id,
            code: admin_code.clone(),
        }),
    )
    .await?;
    let admin_token = admin_verify_response.access_token;
    let admin_claims = auth::verify_jwt(&admin_token)?;
    let admin_user = AuthUser {
        user_id: Uuid::parse_str(&admin_claims.sub)?,
        email: admin_claims.email,
        role: admin_claims.role,
    };

    // 5. Admin creates exactly 5 tasks
    let mut created_task_ids = Vec::new();
    for i in 1..=5 {
        let payload = CreateTaskRequest {
            title: format!("Task {}", i),
            description: Some(format!("Description {}", i)),
            priority: match i {
                1 => "high".to_string(),
                2 => "medium".to_string(),
                3 => "low".to_string(),
                _ => "medium".to_string(),
            },
        };
        let (status, task_response) = handlers::create_task(State(state.clone()), admin_user.clone(), Json(payload)).await?;
        assert_eq!(status, http::StatusCode::CREATED);
        created_task_ids.push(task_response.id);
    }
    assert_eq!(created_task_ids.len(), 5);

    // 6. Assign exactly 3 tasks to James Bond
    let assign_response = handlers::assign_tasks(
        State(state.clone()),
        admin_user.clone(),
        Json(AssignTasksRequest {
            task_ids: vec![created_task_ids[0], created_task_ids[1], created_task_ids[2]],
            assign_to_email: "jamesbond@example.com".to_string(),
        }),
    )
    .await?;
    assert_eq!(assign_response.0["assigned_count"].as_u64().unwrap(), 3);

    // 7. James Bond login starts 2FA
    let james_login_response = handlers::auth_login(
        State(state.clone()),
        Json(LoginRequest {
            email: "jamesbond@example.com".to_string(),
            password: "james123".to_string(),
        }),
    )
    .await?;
    let james_challenge_id = james_login_response.login_challenge_id;

    // 8. Retrieve James Bond verification code
    let email_log = handlers::get_latest_email_log(State(state.clone())).await?.0;
    let james_code = email_log.code.expect("James Bond verification code missing");

    let james_verify_response = handlers::verify_2fa(
        State(state.clone()),
        Json(Verify2FARequest {
            login_challenge_id: james_challenge_id,
            code: james_code,
        }),
    )
    .await?;
    let james_token = james_verify_response.access_token;
    let james_claims = auth::verify_jwt(&james_token)?;
    let james_user = AuthUser {
        user_id: Uuid::parse_str(&james_claims.sub)?,
        email: james_claims.email,
        role: james_claims.role,
    };

    // 9. James Bond cannot create a task
    let james_task_create = handlers::create_task(
        State(state.clone()),
        james_user.clone(),
        Json(CreateTaskRequest {
            title: "Unauthorized Task".to_string(),
            description: Some("Should fail".to_string()),
            priority: "low".to_string(),
        }),
    )
    .await;
    assert!(matches!(james_task_create, Err(task_api::error::ApiError::Forbidden)));

    // 10. James Bond views assigned tasks first time
    let view_response = handlers::view_my_tasks(State(state.clone()), james_user.clone()).await?;
    assert_eq!(view_response.user.email, "jamesbond@example.com");
    assert_eq!(view_response.user.role, "staff");
    assert_eq!(view_response.tasks.len(), 3);
    for task in &view_response.tasks {
        assert_eq!(task.assigned_to.as_deref(), Some("jamesbond@example.com"));
        assert_eq!(task.status, "todo");
    }
    assert_eq!(view_response.summary.total_assigned_tasks, 3);
    assert!(!view_response.cache.hit);

    // 11. Same endpoint again should use cache
    let view_response2 = handlers::view_my_tasks(State(state.clone()), james_user).await?;
    assert!(view_response2.cache.hit);

    Ok(())
}
