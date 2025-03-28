use actix_web::{HttpResponse, web};
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::auth::{generate_token, verify_password};
use crate::db::repositories::UserRepository;
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
    pub user_id: i32,
    pub username: String,
    pub role: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login));
}

async fn login(
    db: web::Data<DbConn>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    process_json_validation(&req)?;

    let repo = UserRepository::new(Arc::new(db.get_ref().clone()));

    let user = match repo.find_by_username(req.username.clone()).await? {
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

    let token = generate_token(&user)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        role: user.role,
    }))
}
