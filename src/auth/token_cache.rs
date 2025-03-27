use dashmap::DashMap;
use log;
use once_cell::sync::Lazy;
use sea_orm::DbConn;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::db::repositories::RefreshTokenRepository;
use crate::error::AppError;

/// Cached data structure: (revoked, last verification timestamp)
type CacheEntry = (bool, Instant);

/// TTL (Time-to-live) for cache entries: 1 minute
pub const CACHE_TTL: Duration = Duration::from_secs(60);

/// Global cache for revoked tokens, indexed by user_id
static TOKEN_CACHE: Lazy<DashMap<i32, CacheEntry>> = Lazy::new(|| {
    log::info!("Initializing token revocation cache");
    DashMap::new()
});

pub fn cache_valid_token(user_id: i32) {
    log::info!("Caching valid token for user_id: {}", user_id);
    TOKEN_CACHE.insert(user_id, (false, Instant::now()));
}

/// Checks if a user's token is revoked, first consulting the cache
/// and then the database if necessary.
///
/// # Arguments
/// * `user_id` - ID of the user whose token is to be verified
/// * `db` - Database connection to query if the cache does not have the information
///
/// # Return
/// * `Ok(true)` if the token is revoked
/// * `Ok(false)` if the token is valid
/// * `Err` if an error occurs while querying the database
pub async fn is_token_revoked(user_id: i32, db: &DbConn) -> Result<bool, AppError> {
    // First, check the cache
    if let Some(entry) = TOKEN_CACHE.get(&user_id) {
        let (revoked, timestamp) = *entry;

        // If the entry is still fresh, use it
        if timestamp.elapsed() < CACHE_TTL {
            log::debug!("Token cache hit for user_id: {}", user_id);
            return Ok(revoked);
        }

        // If expired, remove it to refresh
        log::debug!("Token cache expired for user_id: {}", user_id);
        TOKEN_CACHE.remove(&user_id);
    }

    // Not found in cache or cache expired, query the database
    log::debug!(
        "Token cache miss for user_id: {}, querying database",
        user_id
    );
    let refresh_token_repository = RefreshTokenRepository::new(Arc::new(db.clone()));

    let refresh_token = refresh_token_repository.find_by_user_id(user_id).await?;

    // Determine if it is revoked
    let is_revoked = match refresh_token {
        Some(token) => token.revoked_on.is_some(),
        None => true, // If no token exists, consider it revoked
    };

    // Update the cache with the new information
    TOKEN_CACHE.insert(user_id, (is_revoked, Instant::now()));

    log::debug!(
        "Updated token cache for user_id: {}, revoked: {}",
        user_id,
        is_revoked
    );
    Ok(is_revoked)
}

/// Explicitly marks a token as revoked in the cache
pub fn set_token_revoked(user_id: i32) {
    log::info!(
        "Explicitly marking token as revoked for user_id: {}",
        user_id
    );
    TOKEN_CACHE.insert(user_id, (true, Instant::now()));
}

/// Invalidates the cache entry for a specific user
pub fn invalidate_cache(user_id: i32) {
    log::info!("Invalidating token cache for user_id: {}", user_id);
    TOKEN_CACHE.remove(&user_id);
}

/// Clears the entire cache
pub fn clear_cache() {
    log::info!("Clearing all token cache entries");
    TOKEN_CACHE.clear();
}
