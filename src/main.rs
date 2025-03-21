pub mod api;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;

use actix_web::web::ServiceConfig;
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

    let db: DbConn = Database::connect(conn_str)
        .await
        .expect("Error connecting to the database");

    log::info!("Running database migrations...");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    log::info!("Database migrations completed successfully");

    let config = move |cfg: &mut ServiceConfig| {
        api::configure_routes(cfg, db);
    };

    Ok(config.into())
}
