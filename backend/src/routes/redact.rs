//! POST /api/v1/redact - GDPR masking of free text.

use axum::extract::{FromRequest, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::repository::audit;
use crate::services::redaction::{mask_sensitive_data, RedactionCounts};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RedactRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct RedactResponse {
    pub text: String,
    pub counts: RedactionCounts,
    pub total_redacted: usize,
    /// Audit record id (only when persistence is configured).
    pub audit_id: Option<uuid::Uuid>,
}

/// Max accepted payload; keeps pathological inputs from exhausting memory.
const MAX_TEXT_BYTES: usize = 1024 * 1024;

pub async fn handler(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> AppResult<Json<RedactResponse>> {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let Json(req) = Json::<RedactRequest>::from_request(request, &())
        .await
        .map_err(|_| AppError::BadRequest("body must be JSON: {\"text\": \"...\"}".into()))?;

    if req.text.len() > MAX_TEXT_BYTES {
        return Err(AppError::BadRequest(format!(
            "text exceeds maximum size of {MAX_TEXT_BYTES} bytes"
        )));
    }

    let result = mask_sensitive_data(&req.text);

    // Audit trail: metadata only. Failure is logged but does not fail the
    // request - redaction itself already succeeded (availability over audit).
    let mut audit_id = None;
    if let Some(pool) = &state.db {
        match audit::record_redaction(
            pool,
            request_id.as_deref(),
            &req.text,
            result.counts.email,
            result.counts.iban,
            result.counts.phone,
        )
        .await
        {
            Ok(entry) => audit_id = Some(entry.id),
            Err(err) => tracing::error!(%err, "failed to persist redaction audit entry"),
        }
    }

    Ok(Json(RedactResponse {
        text: result.text,
        total_redacted: result.counts.total(),
        counts: result.counts,
        audit_id,
    }))
}
