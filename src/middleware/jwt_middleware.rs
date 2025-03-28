use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use futures::future::{Ready, ready};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::auth::jwt::{Claims, UserRole, validate_token};

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
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

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
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
        let token_owned = token.to_owned();

        Box::pin(async move {
            let token_data = match validate_token(&token_owned).await {
                Ok(data) => data,
                Err(e) => {
                    log::warn!("Token validation failed: {:?}", e);
                    return Err(ErrorUnauthorized("Invalid or expired token"));
                }
            };

            req.extensions_mut().insert(token_data.claims);
            service.call(req).await
        })
    }
}

pub fn has_role(req: &ServiceRequest, required_role: UserRole) -> Result<bool, Error> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        Ok(claims.role.to_lowercase() == required_role.as_str())
    } else {
        Err(ErrorUnauthorized("User not authenticated"))
    }
}

pub fn require_admin(req: &ServiceRequest) -> Result<bool, Error> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        Ok(claims.is_admin())
    } else {
        Err(ErrorUnauthorized("User not authenticated"))
    }
}

pub fn require_user(req: &ServiceRequest) -> Result<bool, Error> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        Ok(claims.is_user_or_above())
    } else {
        Err(ErrorUnauthorized("User not authenticated"))
    }
}
