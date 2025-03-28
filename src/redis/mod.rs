use once_cell::sync::Lazy;
use redis::{aio::MultiplexedConnection, Client, RedisError};
use std::env;
use std::sync::Arc;

pub mod token_store;

static REDIS_CLIENT: Lazy<Option<Client>> = Lazy::new(|| {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    match Client::open(redis_url.clone()) {
        Ok(client) => {
            log::info!("Redis client created at {}", redis_url);
            Some(client)
        },
        Err(err) => {
            log::error!("Failed to create Redis client: {}", err);
            None
        }
    }
});

pub fn get_client() -> Option<Arc<Client>> {
    REDIS_CLIENT.as_ref().map(|client| Arc::new(client.clone()))
}

pub async fn get_connection() -> Result<MultiplexedConnection, RedisError> {
    match REDIS_CLIENT.as_ref() {
        Some(client) => client.get_multiplexed_async_connection().await,
        None => Err(RedisError::from((
            redis::ErrorKind::IoError,
            "Redis client not initialized",
        )))
    }
}

pub async fn check_connection() -> bool {
    match get_connection().await {
        Ok(mut conn) => {
            match redis::cmd("PING").query_async::<String>(&mut conn).await {
                Ok(pong) => {
                    log::info!("Redis connection test: {}", pong);
                    true
                },
                Err(err) => {
                    log::error!("Redis PING command failed: {}", err);
                    false
                }
            }
        },
        Err(err) => {
            log::error!("Failed to connect to Redis: {}", err);
            false
        }
    }
}