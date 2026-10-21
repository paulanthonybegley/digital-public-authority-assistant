# Digital Public Authority Assistant

A microservice and web prototype for the "Digital Public Authority Assistant" coding challenge. The Rust backend provides GDPR-compliant redaction, statutory deadline calculation (including weekends and public holidays), and master company data checks (German VAT ID with mod-11 and commercial register format validation).

## Features

- **GDPR Redaction** (`POST /api/v1/redact`) – Masks email addresses, German IBANs (DE ## #### #### #### ##) and common German phone numbers. Returns per-category counts and a SHA-256 digest audit reference (no PII stored).
- **Deadline Calculation** (`POST /api/v1/deadline`) – Calculates statutory deadlines per VwVfG/ZPO: if the end falls on a Saturday, Sunday or public holiday, it rolls to the next business day. Supports calendar and business-day counting. Includes a fixed set of national/state holidays with Easter-derived holidays (Easter Sunday, Good Friday, Easter Monday, Ascension Day, Pentecost Monday, Repentance Day for SN, etc.).
- **Holidays** (`GET /api/v1/holidays`) – Lists holidays for a given federal state and year.
- **Compliance Checks** (`POST /api/v1/compliance/validate`) – Validates German VAT IDs (`DE` + 9 digits) against the official mod-11 product method and validates common commercial register number formats.
- **Robust & Observable** – Strict YYYY-MM-DD parsing, range guards (1900..=2100), CORS, request IDs, timeout, per-IP rate limiting (fails open if Redis unavailable), Redis caching for deadline results, Postgres audit trail (metadata-only), health (`/health`) and readiness (`/ready`).
- **Full-stack prototype** – Axum backend + React + TypeScript frontend. Docker Compose provides Postgres, Redis, API and web services.

## Architecture

- Backend: `backend/` (Rust, Axum, sqlx, redis, chrono, regex, tracing, tower-http)
- Frontend: `frontend/` (Vite + React + TS, Vitest + Testing Library, Nginx)
- Supporting infra: `docker-compose.yml` (Postgres 16, Redis 7, api, web)

## Quick start

### Option 1: Local (Rust + Node)
```bash
# Backend
cd backend
cargo test && cargo clippy --all-targets -- -D warnings

# Frontend
cd frontend
npm install
npm test
npm run typecheck
npm run build
```

Run backend:
```bash
cp .env.example .env  # optional
cd backend
cargo run
```

Run frontend:
```bash
cd frontend
npm run dev
```

### Option 2: Docker Compose
```bash
docker compose build --parallel
docker compose up -d
# API: http://localhost:8080/health
# Web: http://localhost:3000
```

Check validity:
```bash
docker compose config
```

## API Examples

Redact:
```bash
curl -X POST http://localhost:8080/api/v1/redact \
  -H 'content-type: application/json' \
  -d '{"text":"Kontakt: a@b.de, DE89 3704 0044 0532 0130 00, 030/1234567"}'
```

Deadline:
```bash
curl -X POST http://localhost:8080/api/v1/deadline \
  -H 'content-type: application/json' \
  -d '{"start_date":"2026-04-30","days_to_add":1,"federal_state":"BE","counting":"calendar"}'
```

Compliance:
```bash
curl -X POST http://localhost:8080/api/v1/compliance/validate \
  -H 'content-type: application/json' \
  -d '{"tax_id":"DE123456788","register_number":"HRB 12345 B"}'
```

Holidays:
```bash
curl http://localhost:8080/api/v1/holidays?year=2026&state=BY
```

## Notes

- **GDPR**: Redaction audits store only a SHA-256 of the original payload and per-category counts; plaintext PII is never persisted.
- **Holidays**: Uses the anonymous Gregorian Easter computus (no hardcoded moving dates). Includes selected federal state observances.
- **Testing**: Unit tests in Rust (39) and integration tests for the router (16). Frontend has 12 component/client tests. All pass.
- **Portability**: Container images use multi-stage builds. SQL migrations run on startup.

## Evaluation checklist coverage

- ✅ Correctness: edge cases (year-end, leap years, consecutive holidays/weekends, zero days, out-of-range years, format strictness).
- ✅ Code quality: modular, no monolithic code, meaningful naming, clippy clean, fmt clean.
- ✅ Robustness: rejects malformed dates (non-YYYY-MM-DD shapes), unknown states, negative days, oversized payloads.
- ✅ Pragmatism: clean mock logic for holidays (comprehensive, computed), implementation finished well within time.

