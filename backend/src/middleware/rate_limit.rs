//! Fixed-window rate limiting backed by Redis (fails open when Redis is down).

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::state::AppState;

const WINDOW_SECONDS: u64 = 60;

pub async fn rate_limit(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let Some(redis) = state.redis.clone() else {
        return next.run(request).await;
    };

    let client = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string());

    let key = format!("rate_limit:{client}");
    let limit = state.config.rate_limit_per_minute;

    let mut conn = redis.clone();
    let counted: Result<i64, redis::RedisError> =
        redis::cmd("INCR").arg(&key).query_async(&mut conn).await;

    match counted {
        Ok(count) => {
            if count == 1 {
                let _: Result<(), _> = redis::cmd("EXPIRE")
                    .arg(&key)
                    .arg(WINDOW_SECONDS)
                    .query_async(&mut conn)
                    .await;
            }
            if count > limit as i64 {
                let mut response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({ "error": "rate limit exceeded, retry later" })),
                )
                    .into_response();
                response.headers_mut().insert(
                    "retry-after",
                    WINDOW_SECONDS.to_string().parse().expect("static header"),
                );
                return response;
            }
            let mut response = next.run(request).await;
            response.headers_mut().insert(
                "x-ratelimit-limit",
                limit.to_string().parse().expect("static header"),
            );
            if let Ok(remaining) = (limit as i64 - count).max(0).to_string().parse() {
                response
                    .headers_mut()
                    .insert("x-ratelimit-remaining", remaining);
            }
            response
        }
        Err(err) => {
            tracing::warn!(%err, "redis unavailable, rate limiting fails open");
            next.run(request).await
        }
    }
}
