use crate::auth::verify_jwt;
use crate::error::{ApiError, ApiResult};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        let claims = verify_jwt(token)?;

        Ok(AuthUser {
            user_id: Uuid::parse_str(&claims.sub).map_err(|_| ApiError::InvalidToken)?,
            email: claims.email,
            role: claims.role,
        })
    }
}

impl AuthUser {
    pub fn require_admin(&self) -> ApiResult<()> {
        if self.role != "admin" {
            return Err(ApiError::Forbidden);
        }
        Ok(())
    }
}
