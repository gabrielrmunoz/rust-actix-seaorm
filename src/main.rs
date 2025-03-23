pub mod api;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod validators;

use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use sea_orm::{Database, DbConn};
use sea_orm_migration::MigratorTrait;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;

use crate::db::migrations::Migrator;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:{secrets.DATABASE_PASSWORD}@localhost:5432/postgres"
    )]
    conn_str: String,
    #[shuttle_runtime::Secrets] _secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let db: DbConn = Database::connect(conn_str)
        .await
        .expect("Error connecting to the database");

    log::info!("Running database migrations...");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    log::info!("Database migrations completed successfully");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(|config| api::configure_routes(config, db.clone()))
            .wrap(Logger::default())
    })
    .bind(format!(
        "{}:{}",
        app_config.server.host, app_config.server.port
    ))?
    .run()
    .await
}
