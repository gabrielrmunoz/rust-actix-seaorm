use actix_web::{HttpResponse, web};
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::auth::jwt::{generate_claims, generate_refresh_token, generate_token_from_claims};
use crate::auth::password::verify_password;
use crate::auth::token_cache::{insert_cache_valid_token, set_token_revoked};
use crate::db::models::RefreshTokenActiveModel;
use crate::db::repositories::{RefreshTokenRepository, UserRepository};
use crate::error::AppError;
use crate::validators::user_validators::process_json_validation;

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_id: i32,
    pub username: String,
    pub role: String,
}

#[derive(Deserialize, Validate)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
        .route("/logout", web::post().to(logout));
}

async fn login(
    db: web::Data<DbConn>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    process_json_validation(&req)?;

    let user_repository = UserRepository::new(Arc::new(db.get_ref().clone()));

    let user = match user_repository
        .find_by_username(req.username.clone())
        .await?
    {
        Some(user) => user,
        None => return Err(AppError::Unauthorized("Invalid credentials".into())),
    };

    let is_valid = verify_password(&req.password, &user.password)?;
    if !is_valid {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    if user.deleted_on.is_some() {
        return Err(AppError::Unauthorized("Account is disabled".into()));
    }

    let claims = generate_claims(&user);
    let token = generate_token_from_claims(&claims)?;
    let refresh_token = generate_refresh_token();

    let refresh_token_repository = RefreshTokenRepository::new(Arc::new(db.get_ref().clone()));
    let active_model = RefreshTokenActiveModel {
        user_id: Set(user.id),
        refresh_token: Set(refresh_token.clone()),
        created_on: Set(DateTime::<Utc>::from_timestamp(claims.iat as i64, 0)
            .unwrap()
            .naive_utc()),
        revoked_on: Set(None),
        ..Default::default()
    };

    refresh_token_repository.create(active_model).await?;
    insert_cache_valid_token(user.id);

    Ok(HttpResponse::Ok().json(LoginResponse {
        token,
        refresh_token,
        user_id: user.id,
        username: user.username,
        role: user.role,
    }))
}

async fn logout(
    db: web::Data<DbConn>,
    req: web::Json<LogoutRequest>,
) -> Result<HttpResponse, AppError> {
    process_json_validation(&req)?;

    let refresh_token_repository = RefreshTokenRepository::new(Arc::new(db.get_ref().clone()));
    let refresh_token = refresh_token_repository
        .find_by_refresh_token(req.refresh_token.clone())
        .await?;

    if let Some(refresh_token) = refresh_token {
        refresh_token_repository.revoke(refresh_token.id).await?;
        set_token_revoked(refresh_token.user_id);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
