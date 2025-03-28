use serde::Deserialize;
use shuttle_runtime::SecretStore;

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
    pub fn from_secrets(secrets: &SecretStore) -> Self {
        let host = secrets
            .get("DATABASE_HOST")
            .unwrap_or("127.0.0.1".to_string());
        let port = secrets
            .get("DATABASE_PORT")
            .unwrap_or("5432".to_string())
            .parse()
            .expect("DATABASE_PORT must be a number");
        let database_url = secrets
            .get("DATABASE_URL")
            .expect("DATABASE_URL must be set in Secrets.toml")
            .to_string();

        AppConfig {
            server: ServerConfig { host, port },
            database: DatabaseConfig { url: database_url },
        }
    }
}
