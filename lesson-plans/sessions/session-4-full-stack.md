# Session 4 — From Pure Functions to a Shipped System

**Duration:** 90 minutes
**Primary code:** `backend/src/lib.rs`, `routes/`, `error.rs`, `state.rs`,
`middleware/rate_limit.rs`, `repository/audit.rs`, `frontend/src/`, `Dockerfile`,
`docker-compose.yml`
**Capstone link:** integration + wrap-up  **Lab:** `worksheets/lab-4-integration.md`

## Learning goal (this session)

Learners can explain how the three pure services become an HTTP API, and can name
where each cross-cutting concern (errors, rate limiting, caching, audit,
health/readiness) lives — including why some of them **fail open**.

## Success criteria

By the end, learners can:
1. Trace one request from `main.rs` through the middleware layers to a handler
   and back.
2. Give the correct HTTP status for: bad JSON, invalid state, negative days,
   429, dependency down.
3. Justify storing only a SHA-256 digest in the audit table.

## Prior knowledge to activate (3 min)

Recap: Sessions 1–3 produced `mask_sensitive_data`, `calculate_deadline`, and
`validate_tax_id` — all **pure functions**. Ask: "What is missing between a pure
function and an API that an untrusted citizen can call?"

---

## Sequence

### 0:00–0:12 — Concept: the shape of the app

Open `lib.rs:28-42` (`build_router`). Draw the layer onion (inner → outer):

```
handlers  →  rate_limit  →  timeout  →  CORS  →  TraceLayer  →  request-id
```

Explain each with a one-line "why":
- **rate_limit** (`middleware/rate_limit.rs:14`) — abuse control, before work.
- **timeout** `REQUEST_TIMEOUT` at 10s (`lib.rs:34-37`) — bound slow work.
- **CORS** (`lib.rs:44-59`) — browser access policy.
- **TraceLayer + request-id** (`lib.rs:39-41`) — observability; the id is echoed
  back so a user can quote it (test `responses_carry_request_id_header`,
  `tests/api.rs:245`).

Then `routes/mod.rs:11-19`: the six routes and their handlers. Connect each to a
service from Sessions 1–3.

### 0:12–0:26 — Concept: errors are a type, not a string

`AppError` (`error.rs:11-22`) with `IntoResponse` (`error.rs:41-58`) mapping:

| Variant | HTTP |
|---|---|
| `BadRequest` | 400 |
| `Validation` | 422 |
| `RateLimited` | 429 |
| `DependencyUnavailable` | 503 |
| `Internal` | 500 (detail hidden) |

Show `From<DeadlineError>` (`error.rs:30-39`) — the domain error becomes an HTTP
error *in one place*. Point at the tests `deadline_rejects_bad_date_and_state`
(`tests/api.rs:153`) and `compliance_requires_at_least_one_field`
(`tests/api.rs:211`).

### 0:26–0:40 — Concept: state, and failing open

`AppState` (`state.rs:11-17`) holds `config`, optional `db`, optional `redis`.
`for_tests` (`state.rs:64-70`) sets both to `None` — this is why the integration
tests run with **no infrastructure**.

Then the design decision worth the whole session:
- Rate limiter **fails open** if Redis errors (`rate_limit.rs:76-79`).
- Deadline cache **fails open** too (`routes/deadline.rs:75-92`).
- Audit failure is logged but does not fail the redaction request
  (`routes/redact.rs:51-68`).

Discussion: availability vs. correctness. When is failing open right? (Redaction
itself already succeeded; blocking it would break the primary job.) When would it
be wrong? (If the limiter protected against a security-critical action.)

### 0:40–0:48 — Concept: privacy-preserving audit

`repository/audit.rs`. The table (`migrations/0001_redaction_audit.sql:2-10`)
stores `payload_sha256` and **counts**, never text. `payload_digest`
(`audit.rs:24-26`) is SHA-256 hex (`digest_is_stable_and_hex_encoded`,
`audit.rs:57`). Ask: "Why counts + digest?" (Prove a redaction happened and
detect repeat inputs, without retaining PII — DSGVO Art. 30 record-keeping.)

### 0:48–1:14 — Demo + Lab 4 (pairs)

Teacher: `docker compose up -d`, then walk `api.http` end-to-end while the
learner follows `worksheets/lab-4-integration.md`:
- probes (`/health`, `/ready`),
- one redaction that returns an `audit_id`,
- a deadline that returns `cached:false` then `cached:true` on repeat,
- a compliance check,
- a 404 and a `/ready` check.
Then the DevOps case study from `backend/Dockerfile:7-18`:
**why the image once shipped a 332 KB stub binary** (a dependency-cache trick
built an empty `fn main(){}`; cargo considered it up to date) and how the
`touch` step (`Dockerfile:17`) fixes it. Ask learners to explain the failure.

### 1:14–1:22 — Wrap-up: the whole system in one picture

Revisit `references/architecture.md`. Each learner writes the request path for
`POST /api/v1/redact` in ≤ 6 boxes.

### 1:22–1:30 — Capstone briefing + exit ticket

Hand out `assessments/practical-challenge.md` and `assessments/rubric.md`.
Exit ticket: "The redaction audit stores no text because… The rate limiter fails
open because… A 422 differs from a 400 in this system because…"

## If you are behind (cut here)

Cut the Docker case study to a 2-minute story and the frontend tour entirely;
keep the layer onion, the error table, and the fail-open discussion.

## Materials

- `backend/src/lib.rs`, `error.rs`, `state.rs`
- `backend/src/routes/*`, `middleware/rate_limit.rs`, `repository/audit.rs`
- `backend/Dockerfile`, `docker-compose.yml`, `api.http`
- `frontend/src/{App.tsx,api.ts,types.ts}` and `components/`
- Worksheets: `worksheets/lab-4-integration.md`; Capstone: `assessments/practical-challenge.md`

## Teacher notes

- The **fail-open** discussion is the intellectual peak of the course; budget
  time for it and do not rush it.
- The frontend is deliberately thin (`api.ts` mirrors the JSON shapes in
  `types.ts`); use it to show type contracts, not React technique.
- If Docker is unavailable, run the backend with `cargo run` and demo the same
  requests against `localhost:8080`.
