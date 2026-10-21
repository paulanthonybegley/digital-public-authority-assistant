//! Deadline calculation (VwVfG § 32 / ZPO § 193) with Redis-backed caching.

use axum::extract::{FromRequest, State};
use axum::Json;
use chrono::{Datelike, NaiveDate};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::services::deadline::{calculate_deadline, DayCounting, DeadlineResult, ShiftReason};
use crate::services::holidays::{public_holidays, FederalState};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DeadlineRequest {
    pub start_date: String,
    pub days_to_add: i64,
    pub federal_state: String,
    /// "calendar" (default) or "business".
    #[serde(default)]
    pub counting: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeadlineResponse {
    pub date: NaiveDate,
    pub raw_date: NaiveDate,
    pub shifted: bool,
    pub reason: Option<ShiftReason>,
    pub counting: String,
    pub federal_state: String,
    /// Whether the answer came from the Redis cache.
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedDeadline {
    date: NaiveDate,
    raw_date: NaiveDate,
    shifted: bool,
    reason: Option<ShiftReason>,
    counting: String,
    federal_state: String,
}

const CACHE_TTL_SECONDS: u64 = 3600;

pub async fn handler(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> AppResult<Json<DeadlineResponse>> {
    let Json(req) = Json::<DeadlineRequest>::from_request(request, &())
        .await
        .map_err(|_| {
            AppError::BadRequest(
                "body must be JSON: {\"start_date\": \"YYYY-MM-DD\", \"days_to_add\": 14, \"federal_state\": \"BY\"}"
                    .into(),
            )
        })?;

    let federal_state = FederalState::parse(&req.federal_state).map_err(AppError::Validation)?;
    let counting = match &req.counting {
        Some(raw) => DayCounting::parse(raw).map_err(AppError::Validation)?,
        None => DayCounting::Calendar,
    };

    let cache_key = format!(
        "deadline:{}:{}:{}:{}",
        federal_state.code(),
        req.start_date.trim(),
        req.days_to_add,
        counting.as_str()
    );

    // Serve from cache when possible (fail open on Redis errors).
    if let Some(manager) = &state.redis {
        let mut conn = manager.clone();
        let cached: Result<Option<String>, redis::RedisError> = conn.get(&cache_key).await;
        if let Ok(Some(raw)) = cached {
            if let Ok(entry) = serde_json::from_str::<CachedDeadline>(&raw) {
                return Ok(Json(DeadlineResponse {
                    date: entry.date,
                    raw_date: entry.raw_date,
                    shifted: entry.shifted,
                    reason: entry.reason,
                    counting: entry.counting,
                    federal_state: entry.federal_state,
                    cached: true,
                }));
            }
        }
    }

    let DeadlineResult {
        date,
        raw_date,
        shifted,
        reason,
    } = calculate_deadline(&req.start_date, req.days_to_add, federal_state, counting)?;

    if let Some(manager) = &state.redis {
        let entry = CachedDeadline {
            date,
            raw_date,
            shifted,
            reason,
            counting: counting.as_str().to_string(),
            federal_state: federal_state.code().to_string(),
        };
        if let Ok(payload) = serde_json::to_string(&entry) {
            let mut conn = manager.clone();
            let _: Result<(), _> = conn
                .set_ex::<_, _, ()>(&cache_key, payload, CACHE_TTL_SECONDS)
                .await;
        }
    }

    Ok(Json(DeadlineResponse {
        date,
        raw_date,
        shifted,
        reason,
        counting: counting.as_str().to_string(),
        federal_state: federal_state.code().to_string(),
        cached: false,
    }))
}

/// GET /api/v1/holidays?year=2026&state=BY
pub async fn holidays_handler(
    State(_state): State<AppState>,
    request: axum::extract::Request,
) -> AppResult<Json<serde_json::Value>> {
    let query = request.uri().query().unwrap_or("").to_string();
    let param = |name: &str| {
        query
            .split('&')
            .find_map(|kv| kv.strip_prefix(&format!("{name}=")))
            .map(str::to_string)
    };

    let year = match param("year") {
        Some(raw) => raw
            .parse::<i32>()
            .map_err(|_| AppError::BadRequest("year must be an integer".into()))?,
        None => chrono::Utc::now().date_naive().year(),
    };
    if !(1900..=2100).contains(&year) {
        return Err(AppError::BadRequest(
            "year must be within 1900..=2100".into(),
        ));
    }
    let state_code = param("state").unwrap_or_else(|| "BE".to_string());
    let federal_state = FederalState::parse(&state_code).map_err(AppError::Validation)?;

    let holidays: Vec<_> = public_holidays(year, federal_state)
        .into_iter()
        .map(|(date, name)| serde_json::json!({ "date": date, "name": name }))
        .collect();
    Ok(Json(serde_json::json!({
        "year": year,
        "federal_state": federal_state.code(),
        "holidays": holidays,
    })))
}
