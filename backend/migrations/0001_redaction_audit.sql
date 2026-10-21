-- GDPR audit trail: metadata only, never plaintext personal data.
CREATE TABLE IF NOT EXISTS redaction_audit (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id     TEXT,
    payload_sha256 TEXT NOT NULL,
    email_count    INTEGER NOT NULL CHECK (email_count >= 0),
    iban_count     INTEGER NOT NULL CHECK (iban_count >= 0),
    phone_count    INTEGER NOT NULL CHECK (phone_count >= 0),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_redaction_audit_created_at
    ON redaction_audit (created_at DESC);
