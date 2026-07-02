use crate::error::{AppError, AppResult};
use crate::state::AppState;
use redis::AsyncCommands;

pub async fn check_rate_limit(
    state: &AppState,
    key: &str,
    limit: u32,
    ttl_seconds: usize,
) -> AppResult<bool> {
    let mut conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let count: Option<u32> = conn
        .get(key)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(c) = count {
        if c >= limit {
            return Ok(false);
        }
    }

    let _: () = conn
        .incr(key, 1)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if count.is_none() {
        let _: () = conn
            .expire(key, ttl_seconds as i64)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(true)
}
