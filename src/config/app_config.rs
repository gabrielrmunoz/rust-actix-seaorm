use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse()
            .expect("SERVER_PORT must be a number");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        AppConfig {
            server: ServerConfig { host, port },
            database: DatabaseConfig { url: database_url },
        }
    }
}
