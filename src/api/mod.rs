use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, web};
use sea_orm::DbConn;
mod users;

pub fn configure_routes(cfg: &mut ServiceConfig, db: DbConn) {
    let db_data = web::Data::new(db);

    cfg.app_data(db_data.clone())
        .service(web::scope("/api").configure(|c| users::configure(c)))
        .route("/health", web::get().to(health_check));
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "UP",
        "message": "Service is running"
    }))
}
