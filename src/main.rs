pub mod api;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;

use actix_web::web::ServiceConfig;
use dotenv::dotenv;
use sea_orm::{Database, DbConn};
use shuttle_actix_web::ShuttleActixWeb;

use crate::config::AppConfig;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenv().ok();

    let app_config = AppConfig::from_env();

    log::info!(
        "Starting server at {}:{}",
        app_config.server.host,
        app_config.server.port
    );

    let db: DbConn = Database::connect(&app_config.database.url)
        .await
        .expect("Error connecting to the database");

    let config = move |cfg: &mut ServiceConfig| {
        api::configure_routes(cfg, db);
    };

    Ok(config.into())
}
