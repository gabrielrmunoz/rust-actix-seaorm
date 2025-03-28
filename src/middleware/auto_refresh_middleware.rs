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
pub struct AutoRefreshMiddleware {
    pub db: Arc<DbConn>,
}

impl AutoRefreshMiddleware {
    pub fn new(db: DbConn) -> Self {
        Self { db: Arc::new(db) }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AutoRefreshMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AutoRefreshMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AutoRefreshMiddlewareService {
            service: Arc::new(service),
            db: self.db.clone(),
        }))
    }
}

pub struct AutoRefreshMiddlewareService<S> {
    service: Arc<S>,
    db: Arc<DbConn>,
}

impl<S, B> Service<ServiceRequest> for AutoRefreshMiddlewareService<S>
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
        // Skip specific routes
        if req.path() == "/api/auth/login"
            || req.path() == "/api/auth/register"
            || req.path() == "/api/auth/refresh"
            || req.path() == "/health"
        {
            return Box::pin(self.service.call(req));
        }

        // Check if authorization header exists
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => return Box::pin(self.service.call(req)),
        };

        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => return Box::pin(self.service.call(req)),
        };

        if !auth_str.starts_with("Bearer ") {
            return Box::pin(self.service.call(req));
        }

        let token = auth_str.trim_start_matches("Bearer ").trim().to_string();
        let db = self.db.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // 1. Try to validate the token normally
            match validate_token(&token).await {
                // If token is valid, continue with the request
                Ok(token_data) => {
                    req.extensions_mut().insert(token_data.claims);
                    service.call(req).await
                }
                // If token is invalid (expired or not found in Redis)
                Err(err) => {
                    log::debug!("Token validation failed: {:?}", err);

                    // 2. Extract user_id from JWT token without validating expiration
                    match extract_user_id_from_token(&token) {
                        Some(user_id) => {
                            log::debug!("Extracted user_id from token: {}", user_id);

                            // 3. Look for a valid refresh token for this user
                            let refresh_token_repository = RefreshTokenRepository::new(&db);

                            let refresh_token_result =
                                refresh_token_repository.find_by_user_id(user_id).await;

                            match refresh_token_result {
                                Ok(Some(refresh_token)) => {
                                    let now = chrono::Utc::now().naive_utc();

                                    // 4. Verify if the refresh token is valid
                                    if refresh_token.revoked_on.is_none()
                                        && refresh_token.expires_on > now
                                    {
                                        log::debug!(
                                            "Found valid refresh token for user {}",
                                            user_id
                                        );

                                        // 5. Use auth::refresh_token to get a new token
                                        match auth::refresh_token(&db, &refresh_token.refresh_token)
                                            .await
                                        {
                                            Ok((new_token, new_claims)) => {
                                                // 6. Insert claims into request extensions
                                                req.extensions_mut().insert(new_claims);

                                                // 7. Update authorization header with new token
                                                match header::HeaderValue::from_str(&format!(
                                                    "Bearer {}",
                                                    new_token
                                                )) {
                                                    Ok(header_value) => {
                                                        req.headers_mut().insert(
                                                            header::AUTHORIZATION,
                                                            header_value,
                                                        );
                                                    }
                                                    Err(e) => {
                                                        log::error!(
                                                            "Failed to create header value: {:?}",
                                                            e
                                                        );
                                                    }
                                                }

                                                // 8. Process the request and add the new token to the response
                                                match service.call(req).await {
                                                    Ok(mut res) => {
                                                        // Add the new token to the response for the client to update
                                                        match header::HeaderValue::from_str(
                                                            &new_token,
                                                        ) {
                                                            Ok(header_value) => {
                                                                res.headers_mut().insert(
                                                                    header::HeaderName::from_static(
                                                                        "x-refresh-token",
                                                                    ),
                                                                    header_value,
                                                                );
                                                            }
                                                            Err(e) => {
                                                                log::error!(
                                                                    "Failed to create response header value: {:?}",
                                                                    e
                                                                );
                                                            }
                                                        }

                                                        log::info!(
                                                            "Auto-refreshed token for user {}",
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
                                                log::error!("Failed to refresh token: {:?}", e);
                                                Err(ErrorUnauthorized(e.to_string()))
                                            }
                                        }
                                    } else {
                                        log::warn!(
                                            "Refresh token for user {} is revoked or expired",
                                            user_id
                                        );
                                        Err(ErrorUnauthorized("Refresh token expired or revoked"))
                                    }
                                }
                                Ok(None) => {
                                    log::warn!("No refresh token found for user {}", user_id);
                                    Err(ErrorUnauthorized("No refresh token found"))
                                }
                                Err(e) => {
                                    log::error!(
                                        "Error finding refresh token for user {}: {:?}",
                                        user_id,
                                        e
                                    );
                                    Err(ErrorUnauthorized("Error finding refresh token"))
                                }
                            }
                        }
                        None => {
                            log::warn!("Could not extract user_id from token");
                            Err(ErrorUnauthorized("Invalid token format"))
                        }
                    }
                }
            }
        })
    }
}

// Extract user_id from token without validating expiration
fn extract_user_id_from_token(token: &str) -> Option<i32> {
    let mut validation = Validation::default();
    validation.validate_exp = false; // Don't validate expiration

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
