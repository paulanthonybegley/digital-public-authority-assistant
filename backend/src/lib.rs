//! Digital Public Authority Assistant - library root.
//!
//! Exposes the HTTP router and the pure domain services so that integration
//! tests (and the binary) share one construction path.

pub mod config;
pub mod error;
pub mod middleware;
pub mod repository;
pub mod routes;
pub mod services;
pub mod state;

use axum::http::{HeaderValue, Method, StatusCode};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::middleware::rate_limit::rate_limit;
use crate::state::AppState;

/// Build the full application router for a given state.
///
/// Layer order (inner to outer): rate limit, timeout, CORS, tracing,
/// request-id propagation, request-id creation.
pub fn build_router(state: AppState) -> Router {
    let cors = build_cors(&state.config.cors_allow_origin);

    Router::new()
        .merge(routes::api_router(state.clone()))
        .layer(axum::middleware::from_fn_with_state(state, rate_limit))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(10),
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}

fn build_cors(allow_origin: &str) -> CorsLayer {
    if allow_origin == "*" {
        CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(tower_http::cors::Any)
    } else {
        let origin = allow_origin
            .parse::<HeaderValue>()
            .expect("CORS_ALLOW_ORIGIN must be a valid header value");
        CorsLayer::new()
            .allow_origin(origin)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([axum::http::header::CONTENT_TYPE])
    }
}

/// Convenience for tests: router with no backing services.
pub fn test_router() -> Router {
    build_router(AppState::for_tests(config::Config::default()))
}
