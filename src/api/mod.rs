use crate::auth::jwt::{JwtMiddleware, RoleGuard};
use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, web};
use sea_orm::DbConn;

mod auth;
mod users;

pub fn configure_routes(cfg: &mut ServiceConfig, db: DbConn) {
    let db_data = web::Data::new(db);

    cfg.app_data(db_data.clone())
        .route("/health", web::get().to(health_check))
        .service(
            web::scope("/api")
                .service(web::scope("/auth").configure(|c| auth::configure(c)))
                .service(web::scope("/register").configure(|c| users::configure_public(c)))
                .service(
                    web::scope("").wrap(JwtMiddleware).service(
                        web::scope("/users")
                            .wrap(RoleGuard::user())
                            .configure(|c| users::configure_protected(c)),
                    ),
                ),
        );
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "UP",
        "message": "Service is running"
    }))
}
