use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::ErrorUnauthorized;
use actix_web::http::header;
use actix_web::{Error, HttpMessage};
use futures::future::{Ready, ready};
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::DbConn;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::api::auth;
use crate::auth::jwt::{Claims, JWT_SECRET, validate_token};
use crate::db::repositories::RefreshTokenRepository;
use crate::redis::get_connection;
use crate::redis::token_store::user_exists;

pub struct AuthMiddleware {
    pub db: Arc<DbConn>,
}

impl AuthMiddleware {
    pub fn new(db: DbConn) -> Self {
        Self { db: Arc::new(db) }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Arc::new(service),
            db: self.db.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
    db: Arc<DbConn>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // Check if the authorization header exists
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => {
                return Box::pin(async {
                    Err(ErrorUnauthorized("Authorization header not found"))
                });
            }
        };

        let auth_str = match auth_header.to_str() {
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

        let token = auth_str.trim_start_matches("Bearer ").trim().to_string();
        let db = self.db.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // 1. Try to validate the token
            match validate_token(&token).await {
                // If the token is valid, proceed with the request
                Ok(token_data) => {
                    req.extensions_mut().insert(token_data.claims);
                    service.call(req).await
                }
                // If the token is invalid (expired or not found in Redis)
                Err(err) => {
                    log::debug!("Token validation failed: {:?}", err);

                    // 2. Extract user_id from the JWT token without validating expiration
                    match extract_user_id_from_token(&token) {
                        Some(user_id) => {
                            log::debug!("Extracted user_id from token: {}", user_id);

                            // 3. Check if the user exists in Redis (previously logged in)
                            match get_connection().await {
                                Ok(mut conn) => {
                                    match user_exists(&mut conn, user_id).await {
                                        Ok(true) => {
                                            // User exists in Redis but the token has expired
                                            // Search for a valid refresh token for this user
                                            let refresh_token_repository =
                                                RefreshTokenRepository::new(&db);
                                            let refresh_token_result = refresh_token_repository
                                                .find_by_user_id(user_id)
                                                .await;

                                            match refresh_token_result {
                                                Ok(Some(refresh_token)) => {
                                                    let now = chrono::Utc::now().naive_utc();

                                                    // Check if the refresh token is valid
                                                    if refresh_token.revoked_on.is_none()
                                                        && refresh_token.expires_on > now
                                                    {
                                                        log::debug!(
                                                            "Found valid refresh token for user {}",
                                                            user_id
                                                        );

                                                        // Use auth::refresh_token to obtain a new token
                                                        match auth::refresh_token(
                                                            &db,
                                                            &mut conn,
                                                            &refresh_token.refresh_token,
                                                        )
                                                        .await
                                                        {
                                                            Ok((new_token, new_claims)) => {
                                                                // Insert claims into the request extensions
                                                                req.extensions_mut()
                                                                    .insert(new_claims.clone());

                                                                // Update authorization header with the new token
                                                                if let Ok(header_value) =
                                                                    header::HeaderValue::from_str(
                                                                        &format!(
                                                                            "Bearer {}",
                                                                            new_token
                                                                        ),
                                                                    )
                                                                {
                                                                    req.headers_mut().insert(
                                                                        header::AUTHORIZATION,
                                                                        header_value,
                                                                    );
                                                                }

                                                                // Process the request with the new token
                                                                match service.call(req).await {
                                                                    Ok(mut res) => {
                                                                        // Add the new token to the response so the client updates it
                                                                        if let Ok(header_value) =
                                                                            header::HeaderValue::from_str(
                                                                                &new_token,
                                                                            )
                                                                        {
                                                                            res.headers_mut().insert(
                                                                                header::HeaderName::from_static("x-refresh-token"),
                                                                                header_value,
                                                                            );
                                                                        }

                                                                        log::info!(
                                                                            "Token automatically refreshed for user {}",
                                                                            user_id
                                                                        );
                                                                        Ok(res)
                                                                    }
                                                                    Err(e) => {
                                                                        log::error!(
                                                                            "Error processing request after token refresh: {:?}",
                                                                            e
                                                                        );
                                                                        Err(e)
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                log::error!(
                                                                    "Failed to refresh token: {:?}",
                                                                    e
                                                                );
                                                                Err(ErrorUnauthorized(
                                                                    "Error refreshing token",
                                                                ))
                                                            }
                                                        }
                                                    } else {
                                                        log::warn!(
                                                            "Refresh token for user {} is revoked or expired",
                                                            user_id
                                                        );
                                                        Err(ErrorUnauthorized(
                                                            "Refresh token has expired or been revoked. Please log in again.",
                                                        ))
                                                    }
                                                }
                                                Ok(None) => {
                                                    log::warn!(
                                                        "No refresh token found for user {}",
                                                        user_id
                                                    );
                                                    Err(ErrorUnauthorized(
                                                        "No refresh token found. Please log in again.",
                                                    ))
                                                }
                                                Err(e) => {
                                                    log::error!(
                                                        "Error finding refresh token for user {}: {:?}",
                                                        user_id,
                                                        e
                                                    );
                                                    Err(ErrorUnauthorized(
                                                        "Error verifying session. Please log in again.",
                                                    ))
                                                }
                                            }
                                        }
                                        Ok(false) => {
                                            // User does not exist in Redis, needs to log in
                                            log::warn!(
                                                "User {} not found in Redis, needs to log in",
                                                user_id
                                            );
                                            Err(ErrorUnauthorized(
                                                "Session expired. Please log in again.",
                                            ))
                                        }
                                        Err(e) => {
                                            log::error!("Error checking user in Redis: {:?}", e);
                                            Err(ErrorUnauthorized(
                                                "Error verifying session. Please log in again.",
                                            ))
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!(
                                        "Failed to connect to Redis during token refresh: {}",
                                        e
                                    );
                                    Err(ErrorUnauthorized(
                                        "Error connecting to session store. Please try again later.",
                                    ))
                                }
                            }
                        }
                        None => {
                            log::warn!("Could not extract user_id from token");
                            Err(ErrorUnauthorized(
                                "Invalid token format. Please log in again.",
                            ))
                        }
                    }
                }
            }
        })
    }
}

// Extract user_id from the token without validating expiration
fn extract_user_id_from_token(token: &str) -> Option<i32> {
    let mut validation = Validation::default();
    validation.validate_exp = false; // Do not validate expiration

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => Some(token_data.claims.user_id),
        Err(e) => {
            log::error!("Failed to decode token: {:?}", e);
            None
        }
    }
}
