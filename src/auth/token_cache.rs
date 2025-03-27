use dashmap::DashMap;
use log;
use once_cell::sync::Lazy;
use sea_orm::DbConn;
use std::time::{Duration, Instant};

use crate::db::repositories::RefreshTokenRepository;
use crate::error::AppError;

type CacheEntry = (bool, Instant);

pub const CACHE_TTL: Duration = Duration::from_secs(60);

static TOKEN_CACHE: Lazy<DashMap<i32, CacheEntry>> = Lazy::new(|| {
    log::info!("Initializing token revocation cache");
    DashMap::new()
});

pub fn insert_cache_valid_token(user_id: i32) {
    log::info!("Caching valid token for user_id: {}", user_id);
    TOKEN_CACHE.insert(user_id, (false, Instant::now()));
}

pub async fn is_token_revoked(user_id: i32, db: &DbConn) -> Result<bool, AppError> {
    if let Some(entry) = TOKEN_CACHE.get(&user_id) {
        let (revoked, timestamp) = *entry;

        if timestamp.elapsed() < CACHE_TTL {
            log::debug!("Token cache hit for user_id: {}", user_id);
            return Ok(revoked);
        }

        log::debug!("Token cache expired for user_id: {}", user_id);
        TOKEN_CACHE.remove(&user_id);
    }

    log::debug!(
        "Token cache miss for user_id: {}, querying database",
        user_id
    );
    let refresh_token_repository = RefreshTokenRepository::new(db);

    let refresh_token = refresh_token_repository.find_by_user_id(user_id).await?;

    let is_revoked = match refresh_token {
        Some(token) => token.revoked_on.is_some(),
        None => true,
    };

    TOKEN_CACHE.insert(user_id, (is_revoked, Instant::now()));

    log::debug!(
        "Updated token cache for user_id: {}, revoked: {}",
        user_id,
        is_revoked
    );
    Ok(is_revoked)
}

pub fn set_token_revoked(user_id: i32) {
    log::info!(
        "Explicitly marking token as revoked for user_id: {}",
        user_id
    );
    TOKEN_CACHE.insert(user_id, (true, Instant::now()));
}

pub fn invalidate_cache(user_id: i32) {
    log::info!("Invalidating token cache for user_id: {}", user_id);
    TOKEN_CACHE.remove(&user_id);
}

pub fn clear_cache() {
    log::info!("Clearing all token cache entries");
    TOKEN_CACHE.clear();
}
