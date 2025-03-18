use crate::models::Entity as UserEntity;
use actix_web::{HttpResponse, Responder, web};
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::prelude::*;
use sea_orm::sqlx::types::chrono::Local;
use serde::{Deserialize, Serialize};

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

pub async fn get_users(db: web::Data<DbConn>) -> impl Responder {
    let users = UserEntity::find()
        .all(db.get_ref())
        .await
        .unwrap_or_else(|_| vec![]);

    HttpResponse::Ok().json(users)
}

pub async fn get_user(db: web::Data<DbConn>, path: web::Path<i32>) -> impl Responder {
    let user_id = path.into_inner();

    let user = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .unwrap_or(None);

    match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_user(
    db: web::Data<DbConn>,
    item: web::Json<CreateUserRequest>,
) -> impl Responder {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let user = crate::models::ActiveModel {
        username: Set(item.username.clone()),
        first_name: Set(item.first_name.clone()),
        last_name: Set(item.last_name.clone()),
        email: Set(item.email.clone()),
        phone: Set(item.phone.clone()),
        created_on: Set(now.clone()),
        updated_on: Set(now.clone()),
        ..Default::default()
    }
    .insert(db.get_ref())
    .await;

    match user {
        Ok(user) => HttpResponse::Created().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
    item: web::Json<UpdateUserRequest>,
) -> impl Responder {
    let user_id = path.into_inner();
    let user = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .unwrap_or(None);

    match user {
        Some(user) => {
            let mut active_model: crate::models::ActiveModel = user.into();

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

            active_model.updated_on = Set(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

            let result = active_model.update(db.get_ref()).await;

            match result {
                Ok(updated_user) => HttpResponse::Ok().json(updated_user),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_user(db: web::Data<DbConn>, path: web::Path<i32>) -> impl Responder {
    let user_id = path.into_inner();

    let result = UserEntity::delete_by_id(user_id).exec(db.get_ref()).await;

    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
