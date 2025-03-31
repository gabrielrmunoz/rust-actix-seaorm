pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod middleware;
pub mod redis;
pub mod validators;

use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use redis::get_connection;
use redis::token_store::set_max_sessions;
use sea_orm::{Database, DbConn};
use sea_orm_migration::MigratorTrait;
use std::io;

use crate::config::AppConfig;
use crate::db::migrations::Migrator;

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app_config = AppConfig::from_env();

    log::info!(
        "Starting server at {}:{}",
        app_config.server.host,
        app_config.server.port
    );

    if !crate::redis::check_connection().await {
        std::process::exit(1);
    }

    let max_sessions = std::env::var("MAX_ACTIVE_SESSIONS")
        .unwrap_or_else(|_| "3".to_string())
        .parse::<usize>()
        .unwrap_or(3);

    if let Ok(mut conn) = get_connection().await {
        if let Err(e) = set_max_sessions(&mut conn, max_sessions).await {
            log::error!("Failed to set max sessions in Redis: {}", e);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to configure Redis: {}", e),
            ));
        } else {
            log::info!(
                "Set maximum concurrent sessions per user to {}",
                max_sessions
            );
        }
    } else {
        log::error!("Failed to obtain Redis connection for configuration");
        return Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "Redis connection failed during configuration",
        ));
    }

    let db: DbConn = Database::connect(&app_config.database.url)
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
