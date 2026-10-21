mod compliance;
mod deadline;
mod health;
mod redact;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .route("/api/v1/redact", post(redact::handler))
        .route("/api/v1/deadline", post(deadline::handler))
        .route("/api/v1/holidays", get(deadline::holidays_handler))
        .route("/api/v1/compliance/validate", post(compliance::handler))
        .with_state(state)
}
