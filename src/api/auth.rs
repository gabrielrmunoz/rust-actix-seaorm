use actix_web::{HttpResponse, web};
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::jwt::{generate_claims, generate_refresh_token, generate_token_from_claims};
use crate::auth::password::verify_password;
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
}

#[derive(Deserialize, Validate)]
pub struct LogoutRequest {
    pub user_id: i32,
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

    let user_repository = UserRepository::new(db.get_ref());

    let user = match user_repository.find_by_username(&req.username).await? {
        Some(user) => user,
        None => return Err(AppError::Unauthorized("Account not registered".into())),
    };

    let is_valid = verify_password(&req.password, &user.password)?;
    if !is_valid {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    if user.deleted_on.is_some() {
        return Err(AppError::Unauthorized("Account is disabled".into()));
    }

    let refresh_token_repository = RefreshTokenRepository::new(db.get_ref());
    let now = Utc::now().naive_utc();

    let existing_refresh_token = refresh_token_repository.find_by_user_id(user.id).await?;

    let refresh_token = if let Some(token) = existing_refresh_token {
        if token.revoked_on.is_none() && token.expires_on > now {
            log::info!("Reusing existing valid refresh token for user {}", user.id);
            token.refresh_token
        } else {
            log::info!("Existing token is invalid for user {}", user.id);
            create_new_refresh_token(&refresh_token_repository, &user).await?
        }
    } else {
        log::info!("No existing refresh token for user {}", user.id);
        create_new_refresh_token(&refresh_token_repository, &user).await?
    };

    let claims = generate_claims(&user);
    let token = generate_token_from_claims(&claims)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        token,
        refresh_token,
    }))
}

async fn create_new_refresh_token(
    repository: &RefreshTokenRepository<'_>,
    user: &crate::db::models::UserModel,
) -> Result<String, AppError> {
    let claims = generate_claims(user);
    let refresh_token = generate_refresh_token();

    let created_on = DateTime::<Utc>::from_timestamp(claims.iat as i64, 0)
        .unwrap()
        .naive_utc();

    let expires_on = Utc::now()
        .checked_add_signed(chrono::Duration::days(365))
        .unwrap_or(Utc::now())
        .naive_utc();

    let refresh_token_active_model = RefreshTokenActiveModel {
        user_id: Set(user.id),
        refresh_token: Set(refresh_token.clone()),
        created_on: Set(created_on),
        expires_on: Set(expires_on),
        revoked_on: Set(None),
        ..Default::default()
    };

    repository.create(refresh_token_active_model).await?;

    Ok(refresh_token)
}

async fn logout(
    db: web::Data<DbConn>,
    req: web::Json<LogoutRequest>,
) -> Result<HttpResponse, AppError> {
    process_json_validation(&req)?;

    let refresh_token_repository = RefreshTokenRepository::new(db.get_ref());
    let refresh_token = refresh_token_repository
        .find_by_user_id(req.user_id)
        .await?;

    if let Some(refresh_token) = refresh_token {
        refresh_token_repository.revoke(refresh_token.id).await?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
