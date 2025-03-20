use actix_web::{HttpResponse, ResponseError};
use sea_orm::DbErr;
use serde::Serialize;
use std::fmt;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug)]
pub enum AppError {
    Database(DbErr),
    Validation(String),
    NotFound(String),
    Unauthorized(String),
    InternalServerError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(err) => write!(f, "Database error: {}", err),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::InternalServerError => write!(f, "Internal server error"),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Database(err) => {
                log::error!("Database error: {}", err);
                HttpResponse::InternalServerError().json(ErrorResponse {
                    status: "error".into(),
                    message: "An internal error occurred".into(),
                })
            }
            AppError::Validation(msg) => HttpResponse::BadRequest().json(ErrorResponse {
                status: "error".into(),
                message: msg.clone(),
            }),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(ErrorResponse {
                status: "error".into(),
                message: msg.clone(),
            }),
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().json(ErrorResponse {
                status: "error".into(),
                message: msg.clone(),
            }),
            AppError::InternalServerError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    status: "error".into(),
                    message: "An internal error occurred".into(),
                })
            }
        }
    }
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        AppError::Database(err)
    }
}
