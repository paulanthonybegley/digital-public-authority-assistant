//! POST /api/v1/compliance/validate - company master data checks.

use axum::extract::{FromRequest, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::services::compliance::{
    validate_register_number, validate_tax_id, RegisterNumberReport, TaxIdReport,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ComplianceRequest {
    /// German VAT ID, e.g. "DE123456788".
    #[serde(default)]
    pub tax_id: Option<String>,
    /// Commercial register number, e.g. "HRB 12345 B".
    #[serde(default)]
    pub register_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ComplianceResponse {
    pub tax_id: Option<TaxIdReport>,
    pub register_number: Option<RegisterNumberReport>,
    pub valid: bool,
}

pub async fn handler(
    State(_state): State<AppState>,
    request: axum::extract::Request,
) -> AppResult<Json<ComplianceResponse>> {
    let Json(req) = Json::<ComplianceRequest>::from_request(request, &())
        .await
        .map_err(|_| {
            AppError::BadRequest(
                "body must be JSON: {\"tax_id\": \"DE...\", \"register_number\": \"HRB ...\"}"
                    .into(),
            )
        })?;

    let tax_id = req
        .tax_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let register_number = req
        .register_number
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if tax_id.is_none() && register_number.is_none() {
        return Err(AppError::BadRequest(
            "provide at least one of 'tax_id' or 'register_number'".into(),
        ));
    }

    let tax_report = tax_id.map(validate_tax_id);
    let register_report = register_number.map(validate_register_number);
    let valid = tax_report.as_ref().map(|r| r.valid).unwrap_or(true)
        && register_report.as_ref().map(|r| r.valid).unwrap_or(true);

    Ok(Json(ComplianceResponse {
        tax_id: tax_report,
        register_number: register_report,
        valid,
    }))
}
