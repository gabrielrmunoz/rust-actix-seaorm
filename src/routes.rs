use crate::handlers;
use actix_web::web;
use sea_orm::DbConn;

pub fn configure_routes(cfg: &mut web::ServiceConfig, db: DbConn) {
    cfg.app_data(web::Data::new(db)).service(
        web::scope("/api")
            .service(
                web::resource("/users")
                    .get(handlers::get_users)
                    .post(handlers::create_user),
            )
            .service(
                web::resource("/users/{id}")
                    .get(handlers::get_user)
                    .put(handlers::update_user)
                    .delete(handlers::delete_user_physical),
            )
            .service(web::resource("/users/{id}/soft-delete").patch(handlers::delete_user_logical))
            .service(web::resource("/users/{id}/restore").patch(handlers::restore_user)),
    );
}
