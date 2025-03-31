use actix_web::error::{ErrorInternalServerError, ErrorNotFound, ErrorUnprocessableEntity};
use actix_web::{Error, HttpResponse, web};
use log::{info, warn};
use sea_orm::DbConn;
use sea_orm::sqlx::types::chrono::Local;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::hash_password;
use crate::db::models::UserActiveModel;
use crate::db::repositories::UserRepository;
use crate::validators::user_validators::{
    process_json_validation, validate_no_spaces, validate_password, validate_phone, validate_role,
};

use sea_orm::ActiveValue::Set;

pub fn configure_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").get(get_users))
        .service(
            web::resource("/{id}")
                .get(get_user)
                .put(update_user)
                .delete(delete_user_physical),
        )
        .service(web::resource("/{id}/soft-delete").patch(delete_user_logical))
        .service(web::resource("/{id}/restore").patch(restore_user));
}

pub fn configure_public(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").post(create_user));
}

#[derive(Deserialize, Serialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(
        min = 3,
        max = 200,
        message = "Username must be between 3 and 50 characters"
    ))]
    #[validate(custom(function = validate_no_spaces))]
    pub username: String,

    #[validate(custom(function = validate_password))]
    pub password: String,

    #[validate(length(min = 3, max = 20, message = "First name cannot exceed 20 characters"))]
    pub first_name: String,

    #[validate(length(min = 3, max = 20, message = "Last name cannot exceed 20 characters"))]
    pub last_name: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(custom(function = validate_phone))]
    pub phone: String,

    #[validate(length(max = 10, message = "Role cannot exceed 10 characters"))]
    #[validate(custom(function = validate_role))]
    pub role: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 20, message = "Username must be at least 3 characters"))]
    #[validate(custom(function = validate_no_spaces))]
    pub username: Option<String>,

    #[validate(length(min = 3, max = 20, message = "First name cannot exceed 20 characters"))]
    pub first_name: Option<String>,

    #[validate(length(min = 3, max = 20, message = "Last name cannot exceed 20 characters"))]
    pub last_name: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    #[validate(custom(function = validate_phone))]
    pub phone: Option<String>,

    #[validate(length(max = 10, message = "Role cannot exceed 10 characters"))]
    #[validate(custom(function = validate_role))]
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct GetUsersParams {
    include_deleted: Option<bool>,
}

pub async fn get_users(
    db: web::Data<DbConn>,
    query: web::Query<GetUsersParams>,
) -> Result<HttpResponse, Error> {
    let include_deleted = query.include_deleted.unwrap_or(false);
    let repo = UserRepository::new(db.get_ref());

    let users = repo
        .find_all(include_deleted)
        .await
        .map_err(|e| ErrorNotFound(format!("Failed to retrieve users: {}", e)))?;

    Ok(HttpResponse::Ok().json(users))
}

pub async fn get_user(db: web::Data<DbConn>, path: web::Path<i32>) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(db.get_ref());

    let user = repo
        .find_by_id(user_id)
        .await
        .map_err(|e| ErrorNotFound(format!("Failed to retrieve user: {}", e)))?;

    match user {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Err(ErrorNotFound(format!("User with ID {} not found", user_id))),
    }
}

pub async fn create_user(
    db: web::Data<DbConn>,
    json_user: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, Error> {
    process_json_validation(&json_user)?;

    info!(
        "Attempting to create user with username: {}",
        json_user.username
    );

    let repo = UserRepository::new(db.get_ref());

    if let Some(_) = repo
        .find_by_username(&json_user.username)
        .await
        .map_err(|e| ErrorInternalServerError(e))?
    {
        return Err(ErrorUnprocessableEntity(format!(
            "Username {} already exists",
            json_user.username
        )));
    }

    if let Some(_) = repo
        .find_by_email(&json_user.email)
        .await
        .map_err(|e| ErrorInternalServerError(e))?
    {
        return Err(ErrorUnprocessableEntity(format!(
            "Email {} already exists",
            json_user.email
        )));
    }

    if let Some(_) = repo
        .find_by_phone(&json_user.phone)
        .await
        .map_err(|e| ErrorInternalServerError(e))?
    {
        return Err(ErrorUnprocessableEntity(format!(
            "Phone {} already exists",
            json_user.phone
        )));
    }

    let now = Local::now().naive_local();
    let hashed_password = hash_password(&json_user.password)?;

    let user = json_user.into_inner();

    let user_model = UserActiveModel {
        username: Set(user.username),
        password: Set(hashed_password),
        first_name: Set(user.first_name),
        last_name: Set(user.last_name),
        email: Set(user.email),
        phone: Set(user.phone),
        role: Set(user.role),
        created_on: Set(now),
        updated_on: Set(now),
        ..Default::default()
    };

    let created_user = repo
        .create(user_model)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to create user: {}", e)))?;

    info!("User created with ID: {}", created_user.id);
    Ok(HttpResponse::Created().json(created_user))
}

pub async fn update_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
    json_user: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, Error> {
    process_json_validation(&json_user)?;

    let user_id = path.into_inner();

    info!("Attempting to update user with ID: {}", user_id);

    let repo = UserRepository::new(db.get_ref());

    if let Some(ref username) = json_user.username {
        if username.trim().is_empty() {
            return Err(ErrorUnprocessableEntity("Username cannot be empty"));
        }

        if let Some(existing_user) = repo
            .find_by_username(username)
            .await
            .map_err(|e| ErrorInternalServerError(e))?
        {
            if existing_user.id != user_id {
                return Err(ErrorUnprocessableEntity(format!(
                    "Username {} already exists",
                    username
                )));
            }
        }
    }

    if let Some(ref email) = json_user.email {
        if email.trim().is_empty() {
            return Err(ErrorUnprocessableEntity("Email cannot be empty"));
        }

        if let Some(existing_user) = repo
            .find_by_email(email)
            .await
            .map_err(|e| ErrorInternalServerError(e))?
        {
            if existing_user.id != user_id {
                return Err(ErrorUnprocessableEntity(format!(
                    "Email {} already exists",
                    email
                )));
            }
        }
    }

    let user_data = repo
        .find_by_id(user_id)
        .await
        .map_err(|e| ErrorInternalServerError(e))?;

    match user_data {
        Some(user_data) => {
            let mut user_active_model: UserActiveModel = user_data.into();

            let user = json_user.into_inner();

            if let Some(username) = user.username {
                user_active_model.username = Set(username);
            }
            if let Some(first_name) = user.first_name {
                user_active_model.first_name = Set(first_name);
            }
            if let Some(last_name) = user.last_name {
                user_active_model.last_name = Set(last_name);
            }
            if let Some(email) = user.email {
                user_active_model.email = Set(email);
            }
            if let Some(phone) = user.phone {
                user_active_model.phone = Set(phone);
            }
            if let Some(role) = user.role {
                user_active_model.role = Set(role);
            }

            user_active_model.updated_on = Set(Local::now().naive_local());

            let updated_user = repo
                .update(user_active_model)
                .await
                .map_err(|e| ErrorInternalServerError(format!("Failed to update user: {}", e)))?;

            info!("User with ID {} updated", user_id);
            Ok(HttpResponse::Ok().json(updated_user))
        }
        None => Err(ErrorNotFound(format!("User with ID {} not found", user_id))),
    }
}

pub async fn delete_user_physical(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(db.get_ref());

    info!("Attempting to physically delete user with ID: {}", user_id);

    let user = repo
        .find_by_id(user_id)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;
    if user.is_none() {
        return Err(ErrorNotFound(format!("User with ID {} not found", user_id)));
    }

    let delete_result = repo
        .delete(user_id)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to delete user: {}", e)))?;

    if delete_result.rows_affected > 0 {
        info!("User with ID {} successfully deleted physically", user_id);
        Ok(HttpResponse::NoContent().finish())
    } else {
        warn!("User with ID {} was not deleted (0 rows affected)", user_id);
        Err(ErrorInternalServerError(
            "Failed to delete user (0 rows affected)",
        ))
    }
}

pub async fn delete_user_logical(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(db.get_ref());

    info!("Attempting to logically delete user with ID: {}", user_id);

    let user = repo
        .find_by_id(user_id)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    match user {
        Some(user) => {
            if user.deleted_on.is_some() {
                warn!("User with ID {} is already logically deleted", user_id);
                return Err(ErrorUnprocessableEntity(format!(
                    "User with ID {} is already marked as deleted",
                    user_id
                )));
            }

            let result = repo.soft_delete(user_id).await.map_err(|e| {
                ErrorInternalServerError(format!("Failed to soft delete user: {}", e))
            })?;

            if result.is_some() {
                info!("User with ID {} successfully marked as deleted", user_id);
                Ok(HttpResponse::NoContent().finish())
            } else {
                Err(ErrorInternalServerError("Failed to mark user as deleted"))
            }
        }
        None => Err(ErrorNotFound(format!("User with ID {} not found", user_id))),
    }
}

pub async fn restore_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let repo = UserRepository::new(db.get_ref());

    info!(
        "Attempting to restore logically deleted user with ID: {}",
        user_id
    );

    let user = repo
        .find_by_id(user_id)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    match user {
        Some(user) => {
            if user.deleted_on.is_none() {
                warn!("User with ID {} is not deleted, cannot restore", user_id);
                return Err(ErrorUnprocessableEntity(format!(
                    "User with ID {} is not marked as deleted",
                    user_id
                )));
            }

            let result = repo
                .restore(user_id)
                .await
                .map_err(|e| ErrorInternalServerError(format!("Failed to restore user: {}", e)))?;

            if result.is_some() {
                info!("User with ID {} successfully restored", user_id);
                Ok(HttpResponse::NoContent().finish())
            } else {
                Err(ErrorInternalServerError("Failed to mark user as deleted"))
            }
        }
        None => Err(ErrorNotFound(format!("User with ID {} not found", user_id))),
    }
}
