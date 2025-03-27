use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage, dev};
use chrono::{Duration, Utc};
use futures::future::{Ready, ready};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use uuid::Uuid;

use crate::db::models::UserModel;
use crate::error::AppError;

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
            _ => Err(format!("Unknown rol: {}", s)),
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
    pub refresh_token: String,
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

static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret_for_development_only".into())
});

pub fn generate_claims(user: &UserModel) -> Claims {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let iat = Utc::now().timestamp() as usize;
    let refresh_token = generate_refresh_token();

    Claims {
        sub: user.id.to_string(),
        exp: expiration,
        iat,
        refresh_token,
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

pub fn validate_token(token: &str) -> Result<TokenData<Claims>, AppError> {
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

    Ok(token_data)
}

pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

pub struct JwtMiddleware;

impl<S, B> dev::Transform<S, dev::ServiceRequest> for JwtMiddleware
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse<B>, Error = Error>
        + 'static,
    B: 'static,
{
    type Response = dev::ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            service: Arc::new(service),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Arc<S>,
}

impl<S, B> dev::Service<dev::ServiceRequest> for JwtMiddlewareService<S>
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse<B>, Error = Error>
        + 'static,
    B: 'static,
{
    type Response = dev::ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: dev::ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        if req.path() == "/api/auth/login"
            || req.path() == "/api/auth/register"
            || req.path() == "/health"
        {
            return Box::pin(async move { service.call(req).await });
        }

        let auth_header = req.headers().get("Authorization");
        if auth_header.is_none() {
            return Box::pin(async { Err(ErrorUnauthorized("Authorization header not found")) });
        }

        let auth_str = match auth_header.unwrap().to_str() {
            Ok(s) => s,
            Err(_) => {
                return Box::pin(async {
                    Err(ErrorUnauthorized("Invalid authorization header format"))
                });
            }
        };

        if !auth_str.starts_with("Bearer ") {
            return Box::pin(async {
                Err(ErrorUnauthorized("Invalid authorization header format"))
            });
        }

        let token = auth_str.trim_start_matches("Bearer ").trim();

        let token_data = match validate_token(token) {
            Ok(data) => data,
            Err(_) => {
                return Box::pin(async { Err(ErrorUnauthorized("Invalid or expired token")) });
            }
        };

        req.extensions_mut().insert(token_data.claims);

        Box::pin(async move { service.call(req).await })
    }
}

pub fn has_role(req: &dev::ServiceRequest, required_role: UserRole) -> Result<bool, Error> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        Ok(claims.role.to_lowercase() == required_role.as_str())
    } else {
        Err(ErrorUnauthorized("User not authenticated"))
    }
}

pub fn require_admin(req: &dev::ServiceRequest) -> Result<bool, Error> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        Ok(claims.is_admin())
    } else {
        Err(ErrorUnauthorized("User not authenticated"))
    }
}

pub fn require_user(req: &dev::ServiceRequest) -> Result<bool, Error> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        Ok(claims.is_user_or_above())
    } else {
        Err(ErrorUnauthorized("User not authenticated"))
    }
}

pub struct RoleGuard {
    pub role: UserRole,
}

impl RoleGuard {
    pub fn new(role: UserRole) -> Self {
        Self { role }
    }

    pub fn admin() -> Self {
        Self {
            role: UserRole::Admin,
        }
    }

    pub fn user() -> Self {
        Self {
            role: UserRole::User,
        }
    }

    pub fn guest() -> Self {
        Self {
            role: UserRole::Guest,
        }
    }
}

impl<S, B> dev::Transform<S, dev::ServiceRequest> for RoleGuard
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse<B>, Error = Error>
        + 'static,
    B: 'static,
{
    type Response = dev::ServiceResponse<B>;
    type Error = Error;
    type Transform = RoleGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RoleGuardMiddleware {
            service: Arc::new(service),
            role: self.role.clone(),
        }))
    }
}

pub struct RoleGuardMiddleware<S> {
    service: Arc<S>,
    role: UserRole,
}

impl<S, B> dev::Service<dev::ServiceRequest> for RoleGuardMiddleware<S>
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse<B>, Error = Error>
        + 'static,
    B: 'static,
{
    type Response = dev::ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: dev::ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let required_role = self.role.clone();

        Box::pin(async move {
            let has_permission = if let Some(claims) = req.extensions().get::<Claims>() {
                let user_role_str = claims.role.to_lowercase();
                user_role_str == required_role.as_str()
                    || (required_role == UserRole::User && user_role_str == "admin")
            } else {
                return Err(ErrorUnauthorized("User not authenticated"));
            };

            if has_permission {
                service.call(req).await
            } else {
                Err(ErrorUnauthorized(format!(
                    "Insufficient permissions. Required role: {}",
                    required_role
                )))
            }
        })
    }
}
