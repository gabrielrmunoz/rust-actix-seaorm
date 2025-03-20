use actix_web::{HttpResponse, web};
use log::{info, warn};
use sea_orm::DbConn;
use sea_orm::sqlx::types::chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::models::UserActiveModel;
use crate::db::repositories::UserRepository;
use crate::error::AppError;
use sea_orm::ActiveValue::Set;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(web::resource("").get(get_users).post(create_user))
            .service(
                web::resource("/{id}")
                    .get(get_user)
                    .put(update_user)
                    .delete(delete_user_physical),
            )
            .service(web::resource("/{id}/soft-delete").patch(delete_user_logical))
            .service(web::resource("/{id}/restore").patch(restore_user)),
    );
}

#[derive(Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Deserialize)]
pub struct GetUsersParams {
    include_deleted: Option<bool>,
}

pub async fn get_users(
    db: web::Data<DbConn>,
    query: web::Query<GetUsersParams>,
) -> Result<HttpResponse, AppError> {
    let include_deleted = query.include_deleted.unwrap_or(false);
    let repo = UserRepository::new(Arc::new(db.get_ref().clone()));

    let users = repo.find_all(include_deleted).await?;

    Ok(HttpResponse::Ok().json(users))
}

pub async fn get_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(Arc::new(db.get_ref().clone()));

    let user = repo.find_by_id(user_id).await?;

    match user {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Err(AppError::NotFound(format!(
            "User with ID {} not found",
            user_id
        ))),
    }
}

pub async fn create_user(
    db: web::Data<DbConn>,
    item: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    info!("Attempting to create user with username: {}", item.username);
    let repo = UserRepository::new(Arc::new(db.get_ref().clone()));

    if item.username.trim().is_empty() {
        return Err(AppError::Validation("Username cannot be empty".into()));
    }

    if item.email.trim().is_empty() {
        return Err(AppError::Validation("Email cannot be empty".into()));
    }

    if let Some(_) = repo.find_by_username(&item.username).await? {
        return Err(AppError::Validation(format!(
            "Username {} already exists",
            item.username
        )));
    }

    if let Some(_) = repo.find_by_email(&item.email).await? {
        return Err(AppError::Validation(format!(
            "Email {} already exists",
            item.email
        )));
    }

    let now = Local::now().naive_local();
    let user_model = UserActiveModel {
        username: Set(item.username.clone()),
        first_name: Set(item.first_name.clone()),
        last_name: Set(item.last_name.clone()),
        email: Set(item.email.clone()),
        phone: Set(item.phone.clone()),
        created_on: Set(now),
        updated_on: Set(now),
        ..Default::default()
    };

    let user = repo.create(user_model).await?;

    info!("User created with ID: {}", user.id);
    Ok(HttpResponse::Created().json(user))
}

pub async fn update_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
    item: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(Arc::new(db.get_ref().clone()));

    info!("Attempting to update user with ID: {}", user_id);

    if let Some(ref username) = item.username {
        if username.trim().is_empty() {
            return Err(AppError::Validation("Username cannot be empty".into()));
        }

        if let Some(existing_user) = repo.find_by_username(username).await? {
            if existing_user.id != user_id {
                return Err(AppError::Validation(format!(
                    "Username {} already exists",
                    username
                )));
            }
        }
    }

    if let Some(ref email) = item.email {
        if email.trim().is_empty() {
            return Err(AppError::Validation("Email cannot be empty".into()));
        }

        if let Some(existing_user) = repo.find_by_email(email).await? {
            if existing_user.id != user_id {
                return Err(AppError::Validation(format!(
                    "Email {} already exists",
                    email
                )));
            }
        }
    }

    let user = repo.find_by_id(user_id).await?;

    match user {
        Some(user) => {
            let mut active_model: UserActiveModel = user.into();

            if let Some(username) = &item.username {
                active_model.username = Set(username.clone());
            }
            if let Some(first_name) = &item.first_name {
                active_model.first_name = Set(Some(first_name.clone()));
            }
            if let Some(last_name) = &item.last_name {
                active_model.last_name = Set(Some(last_name.clone()));
            }
            if let Some(email) = &item.email {
                active_model.email = Set(email.clone());
            }
            if let Some(phone) = &item.phone {
                active_model.phone = Set(Some(phone.clone()));
            }

            active_model.updated_on = Set(Local::now().naive_local());

            let updated_user = repo.update(active_model).await?;

            info!("User with ID {} updated", user_id);
            Ok(HttpResponse::Ok().json(updated_user))
        }
        None => Err(AppError::NotFound(format!(
            "User with ID {} not found",
            user_id
        ))),
    }
}

pub async fn delete_user_physical(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(Arc::new(db.get_ref().clone()));

    info!("Attempting to physically delete user with ID: {}", user_id);

    let user = repo.find_by_id(user_id).await?;
    if user.is_none() {
        return Err(AppError::NotFound(format!(
            "User with ID {} not found",
            user_id
        )));
    }

    let delete_result = repo.delete(user_id).await?;

    if delete_result.rows_affected > 0 {
        info!("User with ID {} successfully deleted physically", user_id);
        Ok(HttpResponse::NoContent().finish())
    } else {
        warn!("User with ID {} was not deleted (0 rows affected)", user_id);
        Err(AppError::InternalServerError)
    }
}

pub async fn delete_user_logical(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(Arc::new(db.get_ref().clone()));

    info!("Attempting to logically delete user with ID: {}", user_id);

    let user = repo.find_by_id(user_id).await?;

    match user {
        Some(user) => {
            if user.deleted_on.is_some() {
                warn!("User with ID {} is already logically deleted", user_id);
                return Err(AppError::Validation(format!(
                    "User with ID {} is already marked as deleted",
                    user_id
                )));
            }

            let now = Local::now().naive_local();
            let result = repo.soft_delete(user_id, now).await?;

            if result.is_some() {
                info!("User with ID {} successfully marked as deleted", user_id);
                Ok(HttpResponse::NoContent().finish())
            } else {
                Err(AppError::InternalServerError)
            }
        }
        None => Err(AppError::NotFound(format!(
            "User with ID {} not found",
            user_id
        ))),
    }
}

pub async fn restore_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(Arc::new(db.get_ref().clone()));

    info!(
        "Attempting to restore logically deleted user with ID: {}",
        user_id
    );

    let user = repo.find_by_id(user_id).await?;

    match user {
        Some(user) => {
            if user.deleted_on.is_none() {
                warn!("User with ID {} is not deleted, cannot restore", user_id);
                return Err(AppError::Validation(format!(
                    "User with ID {} is not marked as deleted",
                    user_id
                )));
            }

            let now = Local::now().naive_local();
            let result = repo.restore(user_id, now).await?;

            if result.is_some() {
                info!("User with ID {} successfully restored", user_id);
                Ok(HttpResponse::NoContent().finish())
            } else {
                Err(AppError::InternalServerError)
            }
        }
        None => Err(AppError::NotFound(format!(
            "User with ID {} not found",
            user_id
        ))),
    }
}
