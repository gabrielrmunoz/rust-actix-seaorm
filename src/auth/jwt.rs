use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;

use crate::db::models::UserModel;
use crate::error::AppError;
use crate::redis::get_connection;
use crate::redis::token_store::is_token_valid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

impl UserRole {
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn is_user_or_above(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::User)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Guest => "guest",
        }
    }

    pub fn is_valid_role(role: &str) -> bool {
        matches!(role.to_lowercase().as_str(), "admin" | "user" | "guest")
    }
}

impl FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "user" => Ok(UserRole::User),
            "guest" => Ok(UserRole::Guest),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub user_id: i32,
    pub username: String,
    pub role: String,
}

impl Claims {
    pub fn get_role(&self) -> Result<UserRole, String> {
        self.role.parse::<UserRole>()
    }

    pub fn is_admin(&self) -> bool {
        self.role.to_lowercase() == "admin"
    }

    pub fn is_user_or_above(&self) -> bool {
        matches!(self.role.to_lowercase().as_str(), "admin" | "user")
    }
}

pub static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret_for_development_only".into())
});

pub fn generate_claims(user: &UserModel) -> Claims {
    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(1))
        .expect("valid timestamp")
        .timestamp() as usize;

    let iat = Utc::now().timestamp() as usize;
    let jti = generate_uuid();

    Claims {
        sub: user.id.to_string(),
        exp: expiration,
        iat,
        jti,
        user_id: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
    }
}

pub fn generate_token_from_claims(claims: &Claims) -> Result<String, AppError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|e| {
        log::error!("Error generating token: {}", e);
        AppError::InternalServerError
    })
}

pub async fn extract_claims_from_header(req: &HttpRequest) -> Result<Claims, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("Authorization header not found".into()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid authorization header format".into()))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::Unauthorized(
            "Invalid authorization header format".into(),
        ));
    }

    let token = auth_str.trim_start_matches("Bearer ").trim();

    let token_data = validate_token(token).await?;

    Ok(token_data.claims)
}

pub async fn validate_token(token: &str) -> Result<TokenData<Claims>, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        log::error!("JWT validation error: {}", e);
        AppError::Unauthorized("Invalid token".into())
    })?;

    if !UserRole::is_valid_role(&token_data.claims.role) {
        log::error!("Token contains invalid role: {}", token_data.claims.role);
        return Err(AppError::Unauthorized("Invalid role in token".into()));
    }

    match get_connection().await {
        Ok(mut conn) => match is_token_valid(&mut conn, &token_data.claims.jti).await {
            Ok(is_valid) => {
                if !is_valid {
                    log::warn!("Token with ID {} has been revoked", token_data.claims.jti);
                    return Err(AppError::Unauthorized("Token has been revoked".into()));
                }
            }
            Err(e) => {
                log::error!("Error checking token in Redis: {}", e);
            }
        },
        Err(e) => {
            log::error!("Failed to connect to Redis during token validation: {}", e);
        }
    }

    Ok(token_data)
}

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}
