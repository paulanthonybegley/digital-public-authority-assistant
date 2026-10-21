//! Integration tests: exercise the real axum router via `tower::ServiceExt`.
//! These run without Postgres/Redis (state falls back to no backing services).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use public_authority_api::test_router;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn send(method: &str, uri: &str, body: Value) -> (StatusCode, Value, axum::http::HeaderMap) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = test_router()
        .oneshot(request)
        .await
        .expect("router responds");

    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    (status, value, headers)
}

async fn post(uri: &str, body: Value) -> (StatusCode, Value) {
    let (status, value, _) = send("POST", uri, body).await;
    (status, value)
}

#[tokio::test]
async fn health_is_always_ok() {
    let (status, body, _) = send("GET", "/health", json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn readiness_reports_disabled_dependencies() {
    let (status, body, _) = send("GET", "/ready", json!(null)).await;
    // No dependencies configured and strict mode off -> ready, checks disabled.
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["checks"]["postgres"]["status"], "disabled");
    assert_eq!(body["checks"]["redis"]["status"], "disabled");
}

#[tokio::test]
async fn redact_endpoint_masks_all_categories() {
    let (status, body) = post(
        "/api/v1/redact",
        json!({
            "text": "Kontakt: max@example.de, IBAN DE89 3704 0044 0532 0130 00, Tel 030/1234567"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["counts"]["email"], 1);
    assert_eq!(body["counts"]["iban"], 1);
    assert_eq!(body["counts"]["phone"], 1);
    assert_eq!(body["total_redacted"], 3);
    let text = body["text"].as_str().unwrap();
    assert!(text.contains("[EMAIL_REDACTED]"));
    assert!(text.contains("[IBAN_REDACTED]"));
    assert!(text.contains("[PHONE_REDACTED]"));
    assert!(!text.contains("max@example.de"));
    assert!(body["audit_id"].is_null()); // persistence disabled in tests
}

#[tokio::test]
async fn redact_rejects_non_json_body() {
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/redact")
        .header("content-type", "application/json")
        .body(Body::from("not json"))
        .unwrap();
    let response = test_router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn redact_requires_text_field() {
    let (status, _) = post("/api/v1/redact", json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn redact_accepts_empty_text() {
    let (status, body) = post("/api/v1/redact", json!({ "text": "" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_redacted"], 0);
    assert_eq!(body["text"], "");
}

#[tokio::test]
async fn deadline_shifts_across_weekend() {
    let (status, body) = post(
        "/api/v1/deadline",
        json!({ "start_date": "2026-01-02", "days_to_add": 1, "federal_state": "BE" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["date"], "2026-01-05");
    assert_eq!(body["shifted"], true);
    assert_eq!(body["reason"], "weekend");
    assert_eq!(body["cached"], false);
}

#[tokio::test]
async fn deadline_shifts_across_holiday() {
    let (status, body) = post(
        "/api/v1/deadline",
        json!({ "start_date": "2026-04-30", "days_to_add": 1, "federal_state": "BY" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["date"], "2026-05-04"); // Labour Day + weekend
    assert_eq!(body["reason"], "public_holiday");
}

#[tokio::test]
async fn deadline_supports_business_day_counting() {
    let (status, body) = post(
        "/api/v1/deadline",
        json!({
            "start_date": "2026-01-02",
            "days_to_add": 1,
            "federal_state": "BE",
            "counting": "business"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["date"], "2026-01-05");
    assert_eq!(body["counting"], "business");
}

#[tokio::test]
async fn deadline_rejects_bad_date_and_state() {
    let (status, body) = post(
        "/api/v1/deadline",
        json!({ "start_date": "31.12.2025", "days_to_add": 1, "federal_state": "BE" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("invalid date"));

    let (status, body) = post(
        "/api/v1/deadline",
        json!({ "start_date": "2025-12-31", "days_to_add": 1, "federal_state": "XX" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("unknown federal state"));

    let (status, _) = post(
        "/api/v1/deadline",
        json!({ "start_date": "2025-12-31", "days_to_add": -3, "federal_state": "BE" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn compliance_validates_tax_id_and_register_number() {
    let (status, body) = post(
        "/api/v1/compliance/validate",
        json!({ "tax_id": "DE123456788", "register_number": "HRB 12345 B" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["valid"], true);
    assert_eq!(body["tax_id"]["valid"], true);
    assert_eq!(body["tax_id"]["normalized"], "DE123456788");
    assert_eq!(body["register_number"]["normalized"], "HRB 12345 B");
}

#[tokio::test]
async fn compliance_reports_specific_errors() {
    let (status, body) = post(
        "/api/v1/compliance/validate",
        json!({ "tax_id": "DE123456789" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["valid"], false);
    assert_eq!(body["tax_id"]["error"], "invalid_check_digit");
    assert!(body["register_number"].is_null());
}

#[tokio::test]
async fn compliance_requires_at_least_one_field() {
    let (status, body) = post("/api/v1/compliance/validate", json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("at least one"));
}

#[tokio::test]
async fn holidays_endpoint_lists_state_specific_days() {
    let (status, body, _) = send("GET", "/api/v1/holidays?year=2026&state=BY", json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["federal_state"], "BY");
    let holidays = body["holidays"].as_array().unwrap();
    let names: Vec<&str> = holidays
        .iter()
        .map(|h| h["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Heilige Drei Könige"));
    assert!(names.contains(&"Fronleichnam"));
    assert!(names.contains(&"Allerheiligen"));

    // A state without those must not list them.
    let (status, body, _) = send("GET", "/api/v1/holidays?year=2026&state=NI", json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = body["holidays"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"Heilige Drei Könige"));
    assert!(!names.contains(&"Reformationstag"));
}

#[tokio::test]
async fn responses_carry_request_id_header() {
    let (_, _, headers) = send("POST", "/api/v1/redact", json!({ "text": "hi" })).await;
    assert!(
        headers.contains_key("x-request-id"),
        "expected x-request-id to be propagated"
    );
}

#[tokio::test]
async fn unknown_route_is_404() {
    let (status, _, _) = send("GET", "/api/v1/does-not-exist", json!(null)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
