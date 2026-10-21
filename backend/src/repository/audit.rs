//! GDPR audit trail (Art. 5 Abs. 1 lit. c / Art. 30 DSGVO).
//!
//! Only metadata is persisted: a SHA-256 of the processed payload and the
//! per-category redaction counts. Plaintext personal data never reaches disk.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AuditEntry {
    pub id: Uuid,
    pub request_id: Option<String>,
    pub payload_sha256: String,
    pub email_count: i32,
    pub iban_count: i32,
    pub phone_count: i32,
    pub created_at: DateTime<Utc>,
}

/// SHA-256 hex digest of the original (pre-redaction) text.
pub fn payload_digest(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

/// Insert a redaction audit record. Returns the persisted entry.
pub async fn record_redaction(
    pool: &PgPool,
    request_id: Option<&str>,
    original_text: &str,
    email_count: usize,
    iban_count: usize,
    phone_count: usize,
) -> Result<AuditEntry, sqlx::Error> {
    sqlx::query_as::<_, AuditEntry>(
        r#"INSERT INTO redaction_audit
             (request_id, payload_sha256, email_count, iban_count, phone_count)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, request_id, payload_sha256, email_count, iban_count, phone_count, created_at"#,
    )
    .bind(request_id)
    .bind(payload_digest(original_text))
    .bind(email_count as i32)
    .bind(iban_count as i32)
    .bind(phone_count as i32)
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_is_stable_and_hex_encoded() {
        let d1 = payload_digest("Hello");
        let d2 = payload_digest("Hello");
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 64);
        assert!(d1.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(d1, payload_digest("hello"));
    }
}
