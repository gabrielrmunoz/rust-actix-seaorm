use crate::handlers;
use actix_web::web;
use sea_orm::DbConn;

pub fn configure_routes(cfg: &mut web::ServiceConfig, db: DbConn) {
    cfg.app_data(web::Data::new(db)).service(
        web::scope("/api")
            .route("/users", web::get().to(handlers::get_users))
            .route("/users/{id}", web::get().to(handlers::get_user))
            .route("/users", web::post().to(handlers::create_user))
            .route("/users/{id}", web::put().to(handlers::update_user))
            .route(
                "/users/{id}",
                web::delete().to(handlers::delete_user_physical),
            )
            .route(
                "/users/{id}/soft-delete",
                web::patch().to(handlers::delete_user_logical),
            )
            .route(
                "/users/{id}/restore",
                web::patch().to(handlers::restore_user),
            ),
    );
}
