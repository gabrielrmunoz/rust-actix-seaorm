pub mod api;
pub mod db;

use actix_web::web::ServiceConfig;
use dotenv::dotenv;
use sea_orm::{Database, DbConn};
use shuttle_actix_web::ShuttleActixWeb;
use std::env;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db: DbConn = Database::connect(&database_url)
        .await
        .expect("Error connecting to the database");

    let config = move |cfg: &mut ServiceConfig| {
        api::configure_routes(cfg, db);
    };

    Ok(config.into())
}
