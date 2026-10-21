//! Liveness (`/health`) and readiness (`/ready`) probes.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::state::AppState;

/// Liveness: the process is up. Never touches dependencies.
pub async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// Readiness: backing services reachable. 503 when a configured dependency
/// is down (or, in strict mode, when one is missing entirely).
pub async fn ready(State(state): State<AppState>) -> Response {
    let mut checks = serde_json::Map::new();

    match &state.db {
        Some(pool) => match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_result) => {
                checks.insert("postgres".into(), json!({ "status": "ok" }));
            }
            Err(err) => {
                checks.insert(
                    "postgres".into(),
                    json!({ "status": "error", "detail": err.to_string() }),
                );
            }
        },
        None => {
            let status = if state.config.strict_readiness {
                "error"
            } else {
                "disabled"
            };
            checks.insert(
                "postgres".into(),
                json!({ "status": status, "detail": "DATABASE_URL not configured" }),
            );
        }
    }

    match &state.redis {
        Some(manager) => {
            let mut conn = manager.clone();
            match redis::cmd("PING").query_async::<String>(&mut conn).await {
                Ok(_) => {
                    checks.insert("redis".into(), json!({ "status": "ok" }));
                }
                Err(err) => {
                    checks.insert(
                        "redis".into(),
                        json!({ "status": "error", "detail": err.to_string() }),
                    );
                }
            }
        }
        None => {
            let status = if state.config.strict_readiness {
                "error"
            } else {
                "disabled"
            };
            checks.insert(
                "redis".into(),
                json!({ "status": status, "detail": "REDIS_URL not configured" }),
            );
        }
    }

    // Only hard errors make the service not-ready; a non-strict, disabled
    // dependency is reported but tolerated.
    let healthy = checks.values().all(|v| v["status"] != "error");
    let body = Json(json!({
        "status": if healthy { "ready" } else { "not_ready" },
        "checks": checks,
    }));
    if healthy {
        (StatusCode::OK, body).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, body).into_response()
    }
}
