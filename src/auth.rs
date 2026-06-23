use crate::error::{ApiError, ApiResult};
use crate::models::Claims;
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::SaltString;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use uuid::Uuid;

const JWT_SECRET: &str = "your-secret-key-change-in-production";
const TOKEN_EXPIRY_HOURS: i64 = 24;
const VERIFICATION_CODE_EXPIRY_MINUTES: i64 = 5;

pub fn hash_password(password: &str) -> ApiResult<String> {
    let salt = SaltString::generate(rand::thread_rng());
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| ApiError::InternalServerError)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> ApiResult<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| ApiError::InternalServerError)?;
    
    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect()
}

pub fn hash_verification_code(code: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn generate_jwt(user_id: Uuid, email: String, role: String) -> ApiResult<String> {
    let now = Utc::now();
    let exp = (now + chrono::Duration::hours(TOKEN_EXPIRY_HOURS)).timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email,
        role,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .map_err(|_| ApiError::InvalidToken)
}

pub fn verify_jwt(token: &str) -> ApiResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ApiError::InvalidToken)
}

pub fn get_verification_code_expiry() -> chrono::DateTime<Utc> {
    Utc::now() + chrono::Duration::minutes(VERIFICATION_CODE_EXPIRY_MINUTES)
}
