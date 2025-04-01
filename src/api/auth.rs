use actix_web::error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{Error, HttpRequest, HttpResponse, web};
use chrono::{DateTime, Utc};
use redis::aio::MultiplexedConnection;
use sea_orm::ActiveValue::Set;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::Claims;
use crate::auth::jwt::{
    extract_claims_without_exp_validation, generate_claims, generate_token_from_claims,
    generate_uuid,
};
use crate::auth::password::verify_password;
use crate::db::models::{RefreshTokenActiveModel, UserModel};
use crate::db::repositories::{RefreshTokenRepository, UserRepository};

use crate::redis::get_connection;
use crate::redis::token_store::{
    get_user_sessions_count, register_token, revoke_all_user_tokens, revoke_token,
};
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

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub revoke_all: Option<bool>,
}

#[derive(Deserialize, Validate)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
        .route("/logout", web::post().to(logout));
}

pub async fn refresh_token(
    db: &DbConn,
    redis_conn: &mut MultiplexedConnection,
    refresh_token_str: &str,
) -> Result<(String, Claims), Error> {
    let refresh_token_repository = RefreshTokenRepository::new(db);
    let user_repository = UserRepository::new(db);

    // Find and validate refresh token
    let refresh_token_model = refresh_token_repository
        .find_by_refresh_token(refresh_token_str)
        .await
        .map_err(|_: sea_orm::DbErr| ErrorUnauthorized(String::from("Invalid refresh token")))?
        .ok_or_else(|| ErrorUnauthorized(String::from("Refresh token not found")))?;

    let now = Utc::now().naive_utc();
    if refresh_token_model.revoked_on.is_some() || refresh_token_model.expires_on <= now {
        return Err(ErrorUnauthorized("Refresh token expired or revoked"));
    }

    // Get user information
    let user = user_repository
        .find_by_id(refresh_token_model.user_id)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| ErrorUnauthorized::<String>("User not found".to_string()))?;

    if user.deleted_on.is_some() {
        return Err(ErrorUnauthorized("Account is disabled"));
    }

    // Generate new JWT token
    let claims = generate_claims(&user);
    let token = generate_token_from_claims(&claims)?;
    let expires_in_secs = claims.exp.saturating_sub(claims.iat);

    // Register token in Redis
    if let Err(e) = register_token(redis_conn, user.id, &claims.jti, expires_in_secs).await {
        log::error!("Failed to register token in Redis: {}", e);
        return Err(ErrorUnauthorized("Error registering token in Redis"));
    }

    return Ok((token, claims));
}

async fn login(db: web::Data<DbConn>, req: web::Json<LoginRequest>) -> Result<HttpResponse, Error> {
    process_json_validation(&req)?;

    let user_repository = UserRepository::new(db.get_ref());

    let user = match user_repository
        .find_by_username(&req.username)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?
    {
        Some(user) => user,
        None => return Err(ErrorUnauthorized("Account not registered".to_string())),
    };

    let is_valid = verify_password(&req.password, &user.password)?;
    if !is_valid {
        return Err(ErrorUnauthorized("Invalid credentials"));
    }

    if user.deleted_on.is_some() {
        return Err(ErrorUnauthorized("Account is disabled"));
    }

    let has_active_tokens = match get_connection().await {
        Ok(mut conn) => match get_user_sessions_count(&mut conn, user.id).await {
            Ok(count) => {
                if count > 0 {
                    log::warn!("User already has active sessions");
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                log::error!("Failed to check active sessions: {}", e);
                false
            }
        },
        Err(e) => {
            log::error!("Failed to connect to Redis during login: {}", e);
            false
        }
    };

    if has_active_tokens {
        return Err(ErrorForbidden(
            "You already have an active session. Please logout from other devices first.",
        ));
    }

    let refresh_token_repository = RefreshTokenRepository::new(db.get_ref());
    let now = Utc::now().naive_utc();

    let existing_refresh_token = refresh_token_repository
        .find_by_user_id(user.id)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    let refresh_token = if let Some(refresh_token_active_model) = existing_refresh_token {
        if refresh_token_active_model.revoked_on.is_none()
            && refresh_token_active_model.expires_on > now
        {
            log::info!("Reusing existing valid refresh token for user {}", user.id);
            refresh_token_active_model.refresh_token
        } else {
            log::info!("Existing refresh token is invalid for user {}", user.id);
            create_new_refresh_token(&refresh_token_repository, &user).await?
        }
    } else {
        log::info!("No existing refresh token for user {}", user.id);
        create_new_refresh_token(&refresh_token_repository, &user).await?
    };

    let claims = generate_claims(&user);
    let token = generate_token_from_claims(&claims)?;
    let expires_in_secs = claims.exp.saturating_sub(claims.iat);

    match get_connection().await {
        Ok(mut conn) => {
            if let Err(e) = register_token(&mut conn, user.id, &claims.jti, expires_in_secs).await {
                log::error!("Failed to register token in Redis: {}", e);
            }
        }
        Err(e) => {
            log::error!("Failed to connect to Redis during login: {}", e);
        }
    }

    Ok(HttpResponse::Ok().json(LoginResponse {
        token,
        refresh_token,
    }))
}

async fn create_new_refresh_token(
    repository: &RefreshTokenRepository<'_>,
    user: &UserModel,
) -> Result<String, Error> {
    let claims = generate_claims(user);
    let refresh_token = generate_uuid();

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

    repository
        .create(refresh_token_active_model)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    Ok(refresh_token)
}

async fn logout(
    req: HttpRequest,
    logout_req: web::Json<LogoutRequest>,
) -> Result<HttpResponse, Error> {
    let claims = match extract_claims_without_exp_validation(&req).await {
        Ok(claims) => claims,
        Err(e) => {
            log::warn!(
                "Failed to extract claims during logout: {}. Continuing anyway.",
                e
            );

            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Session terminated (token was already invalid)"
            })));
        }
    };

    let success = match get_connection().await {
        Ok(mut conn) => {
            if logout_req.revoke_all.unwrap_or(false) {
                match revoke_all_user_tokens(&mut conn, claims.user_id).await {
                    Ok(_) => true,
                    Err(e) => {
                        log::error!("Failed to revoke all tokens: {}", e);
                        false
                    }
                }
            } else {
                match revoke_token(&mut conn, &claims.jti).await {
                    Ok(_) => true,
                    Err(e) => {
                        log::error!("Failed to revoke token: {}", e);

                        if e.to_string().contains("not found") {
                            true
                        } else {
                            false
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to connect to Redis during logout: {}", e);
            false
        }
    };

    if success {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Successfully logged out"
        })))
    } else {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Logged out (but there were some issues cleaning up session data)"
        })))
    }
}
