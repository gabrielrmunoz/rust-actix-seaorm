use redis::{AsyncCommands, RedisError, aio::MultiplexedConnection};
use uuid::Uuid;

const TOKEN_PREFIX: &str = "jwt:token:";
const USER_PREFIX: &str = "jwt:user:";
const MAX_SESSIONS_KEY: &str = "jwt:config:max_sessions";

pub async fn register_token(
    conn: &mut MultiplexedConnection,
    user_id: i32,
    jti: &str,
    expires_in_secs: usize,
) -> Result<(), RedisError> {
    let token_key = format!("{}{}", TOKEN_PREFIX, jti);

    let _: () = conn
        .set_ex(
            &token_key,
            user_id.to_string(),
            expires_in_secs.try_into().unwrap(),
        )
        .await?;

    let user_key = format!("{}{}", USER_PREFIX, user_id);
    let _: () = conn.sadd(&user_key, jti).await?;

    let max_sessions: usize = match conn.get(MAX_SESSIONS_KEY).await {
        Ok(val) => val,
        Err(_) => 3,
    };

    enforce_max_sessions(conn, user_id, max_sessions).await?;

    Ok(())
}

pub async fn is_token_valid(
    conn: &mut MultiplexedConnection,
    jti: &str,
) -> Result<bool, RedisError> {
    let token_key = format!("{}{}", TOKEN_PREFIX, jti);
    conn.exists(&token_key).await
}

pub async fn get_token_user_id(
    conn: &mut MultiplexedConnection,
    jti: &str,
) -> Result<Option<i32>, RedisError> {
    let token_key = format!("{}{}", TOKEN_PREFIX, jti);
    let result: Option<String> = conn.get(&token_key).await?;

    match result {
        Some(user_id_str) => match user_id_str.parse::<i32>() {
            Ok(user_id) => Ok(Some(user_id)),
            Err(_) => Ok(None),
        },
        None => Ok(None),
    }
}

pub async fn revoke_token(conn: &mut MultiplexedConnection, jti: &str) -> Result<bool, RedisError> {
    let user_id = match get_token_user_id(conn, jti).await? {
        Some(id) => id,
        None => return Ok(false),
    };

    let token_key = format!("{}{}", TOKEN_PREFIX, jti);
    let _: () = conn.del(&token_key).await?;

    let user_key = format!("{}{}", USER_PREFIX, user_id);
    let removed: i32 = conn.srem(&user_key, jti).await?;

    Ok(removed > 0)
}

pub async fn revoke_all_user_tokens(
    conn: &mut MultiplexedConnection,
    user_id: i32,
) -> Result<usize, RedisError> {
    let user_key = format!("{}{}", USER_PREFIX, user_id);

    let tokens: Vec<String> = conn.smembers(&user_key).await?;
    let count = tokens.len();

    for jti in &tokens {
        let token_key = format!("{}{}", TOKEN_PREFIX, jti);
        let _: () = conn.del(&token_key).await?;
    }

    let _: () = conn.del(&user_key).await?;

    Ok(count)
}

pub async fn user_exists(
    conn: &mut MultiplexedConnection,
    user_id: i32,
) -> Result<bool, RedisError> {
    let user_key = format!("{}{}", USER_PREFIX, user_id);
    conn.exists(&user_key).await
}

pub async fn set_max_sessions(
    conn: &mut MultiplexedConnection,
    max_sessions: usize,
) -> Result<(), RedisError> {
    conn.set(MAX_SESSIONS_KEY, max_sessions.to_string()).await
}

pub async fn get_user_sessions_count(
    conn: &mut MultiplexedConnection,
    user_id: i32,
) -> Result<usize, RedisError> {
    let user_key = format!("{}{}", USER_PREFIX, user_id);
    conn.scard(&user_key).await
}

pub async fn list_user_tokens(
    conn: &mut MultiplexedConnection,
    user_id: i32,
) -> Result<Vec<String>, RedisError> {
    let user_key = format!("{}{}", USER_PREFIX, user_id);
    conn.smembers(&user_key).await
}

async fn enforce_max_sessions(
    conn: &mut MultiplexedConnection,
    user_id: i32,
    max_sessions: usize,
) -> Result<(), RedisError> {
    let user_key = format!("{}{}", USER_PREFIX, user_id);

    let tokens: Vec<String> = conn.smembers(&user_key).await?;

    if tokens.len() <= max_sessions {
        return Ok(());
    }

    let tokens_to_remove = tokens.len() - max_sessions;

    let mut token_ttls: Vec<(String, i64)> = Vec::with_capacity(tokens.len());

    for jti in &tokens {
        let token_key = format!("{}{}", TOKEN_PREFIX, jti);
        let ttl: i64 = conn.ttl(&token_key).await?;
        token_ttls.push((jti.clone(), ttl));
    }

    token_ttls.sort_by_key(|(_jti, ttl)| *ttl);

    for i in 0..tokens_to_remove {
        if i < token_ttls.len() {
            let (jti, _) = &token_ttls[i];
            let token_key = format!("{}{}", TOKEN_PREFIX, jti);

            let _: () = conn.del(&token_key).await?;

            let _: () = conn.srem(&user_key, jti).await?;
        }
    }

    Ok(())
}

pub async fn cleanup_expired_tokens(conn: &mut MultiplexedConnection) -> Result<(), RedisError> {
    let pattern = format!("{}*", USER_PREFIX);
    let user_keys: Vec<String> = conn.keys(&pattern).await?;

    for user_key in user_keys {
        let tokens: Vec<String> = conn.smembers(&user_key).await?;

        for jti in tokens {
            let token_key = format!("{}{}", TOKEN_PREFIX, jti);

            let exists: bool = conn.exists(&token_key).await?;

            if !exists {
                let _: () = conn.srem(&user_key, &jti).await?;
            }
        }
    }

    Ok(())
}

pub fn generate_token_id() -> String {
    Uuid::new_v4().to_string()
}
