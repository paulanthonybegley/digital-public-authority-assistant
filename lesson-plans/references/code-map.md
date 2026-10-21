# Reference — Code Map

A concept → `file:line` index for all four sessions. Line numbers refer to the
state of the repository when this package was written; if the code moves, grep
for the named identifier.

Convention: `path:line` (start of the definition or the quoted test).

---

## Session 1 — Redaction

| Concept | Location |
|---|---|
| Combined regex, ordering, `LazyLock` | `backend/src/services/redaction.rs:11` |
| The pattern (email \| IBAN \| phone) | `backend/src/services/redaction.rs:21-25` |
| Phone constraints (no `00` prefix, ≥5-digit subscriber) | `backend/src/services/redaction.rs:14-24` |
| Placeholder constants | `backend/src/services/redaction.rs:29-31` |
| `RedactionCounts` (+ `total()`) | `backend/src/services/redaction.rs:33-44` |
| `RedactionResult` | `backend/src/services/redaction.rs:46-50` |
| `mask_sensitive_data` (single-pass loop) | `backend/src/services/redaction.rs:53-79` |
| Test: mixed document, per-category counts | `backend/src/services/redaction.rs:129` |
| Test: adjacency does not bleed | `backend/src/services/redaction.rs:148` |
| Test: dates/fragments not masked | `backend/src/services/redaction.rs:176` |
| Test (regression): file numbers before dates | `backend/src/services/redaction.rs:184` |
| Test: short-but-valid national numbers | `backend/src/services/redaction.rs:205` |
| Route handler `POST /api/v1/redact` | `backend/src/routes/redact.rs:29` |
| Payload size guard (1 MiB) | `backend/src/routes/redact.rs:27,43` |

**The bug to teach:** the national rule `\b0\d{2,4}[\s\-/]\d{3,}` matched
`0042 2026`. Fix and rationale live at `redaction.rs:14-24`; the test that locks
it in is `redaction.rs:184`.

---

## Session 2 — Deadlines & holidays

| Concept | Location |
|---|---|
| `DayCounting` (Calendar / Business) | `backend/src/services/deadline.rs:10-36` |
| `DeadlineError` (typed failures) | `backend/src/services/deadline.rs:38-46` |
| `ShiftReason` (Weekend / PublicHoliday) | `backend/src/services/deadline.rs:49-54` |
| `DeadlineResult` | `backend/src/services/deadline.rs:56-66` |
| Strict `YYYY-MM-DD` parsing + range guard | `backend/src/services/deadline.rs:69-88` |
| `calculate_deadline` | `backend/src/services/deadline.rs:96-159` |
| Calendar branch (add then roll) | `backend/src/services/deadline.rs:108-118` |
| Business branch (count business days) | `backend/src/services/deadline.rs:119-134` |
| `shifted` / `reason` derivation | `backend/src/services/deadline.rs:137-151` |
| Test: weekday stays put | `backend/src/services/deadline.rs:170` |
| Test: Friday → Monday | `backend/src/services/deadline.rs:179` |
| Test: holiday rolls forward | `backend/src/services/deadline.rs:188` |
| Test: consecutive holiday/weekend bridge | `backend/src/services/deadline.rs:196` |
| Test: year-end transitions | `backend/src/services/deadline.rs:206` |
| Test: leap day | `backend/src/services/deadline.rs:227` |
| Test: invalid-date shapes | `backend/src/services/deadline.rs:266` |
| `FederalState` enum + parse | `backend/src/services/holidays.rs:10-54` |
| Easter computus (anonymous Gregorian) | `backend/src/services/holidays.rs:78-94` |
| `repentance_day` (Wed before 23 Nov) | `backend/src/services/holidays.rs:97-103` |
| `public_holidays` (fixed + Easter-derived + state) | `backend/src/services/holidays.rs:106-171` |
| `is_public_holiday` | `backend/src/services/holidays.rs:174-178` |
| `is_business_day` | `backend/src/services/holidays.rs:181-183` |
| Test: Easter known years | `backend/src/services/holidays.rs:190` |
| Test: Reformation Day exceptions | `backend/src/services/holidays.rs:237` |
| Test: Repentance Day | `backend/src/services/holidays.rs:247` |
| Route handler `POST /api/v1/deadline` | `backend/src/routes/deadline.rs:48` |
| Route handler `GET /api/v1/holidays` | `backend/src/routes/deadline.rs:130` |
| Redis cache (GET/SET, TTL, fail-open) | `backend/src/routes/deadline.rs:75-116` |

---

## Session 3 — Compliance

| Concept | Location |
|---|---|
| `TaxIdError` (Empty/InvalidFormat/InvalidCheckDigit) | `backend/src/services/compliance.rs:8-24` |
| `TaxIdReport` | `backend/src/services/compliance.rs:26-33` |
| `normalize_tax_id` | `backend/src/services/compliance.rs:36-42` |
| `compute_check_digit` (mod-11 product method) | `backend/src/services/compliance.rs:47-63` |
| The forbidden-zero rule | `backend/src/services/compliance.rs:62` |
| `validate_tax_id` | `backend/src/services/compliance.rs:66-96` |
| `RegisterNumberError` | `backend/src/services/compliance.rs:99-115` |
| `RegisterNumberReport` | `backend/src/services/compliance.rs:117-123` |
| `validate_register_number` (structural) | `backend/src/services/compliance.rs:131-199` |
| Accepted prefixes + canonical display | `backend/src/services/compliance.rs:157-172` |
| Test: known valid VAT ID | `backend/src/services/compliance.rs:206` |
| Test: check digit 0 invalid | `backend/src/services/compliance.rs:214` |
| Test: wrong check digit | `backend/src/services/compliance.rs:222` |
| Test: normalization + case | `backend/src/services/compliance.rs:229` |
| Test: format errors | `backend/src/services/compliance.rs:236` |
| Test: fuzz never panics | `backend/src/services/compliance.rs:255` |
| Test: register valid/invalid shapes | `backend/src/services/compliance.rs:263,280` |
| Route handler `POST /api/v1/compliance/validate` | `backend/src/routes/compliance.rs:30` |

---

## Session 4 — System

| Concept | Location |
|---|---|
| `build_router` (layer ordering) | `backend/src/lib.rs:28-42` |
| CORS construction | `backend/src/lib.rs:44-59` |
| `test_router` helper | `backend/src/lib.rs:62-64` |
| Route table | `backend/src/routes/mod.rs:11-19` |
| `AppError` variants | `backend/src/error.rs:11-22` |
| Domain → HTTP error mapping | `backend/src/error.rs:30-39` |
| `IntoResponse` (status table) | `backend/src/error.rs:41-58` |
| `AppState` (config, db?, redis?) | `backend/src/state.rs:11-17` |
| `AppState::new` (connect + migrate) | `backend/src/state.rs:20-61` |
| `AppState::for_tests` (no infra) | `backend/src/state.rs:64-70` |
| Rate limiter (fixed window) | `backend/src/middleware/rate_limit.rs:14-81` |
| Fail-open on Redis error | `backend/src/middleware/rate_limit.rs:76-79` |
| Rate-limit response headers | `backend/src/middleware/rate_limit.rs:52-73` |
| Audit `payload_digest` (SHA-256) | `backend/src/repository/audit.rs:24-26` |
| Audit `record_redaction` | `backend/src/repository/audit.rs:29-50` |
| Audit table schema | `backend/migrations/0001_redaction_audit.sql:2-10` |
| Liveness `/health` | `backend/src/routes/health.rs:12` |
| Readiness `/ready` (+ disabled tolerance) | `backend/src/routes/health.rs:18,74` |
| Config from env | `backend/src/config.rs:38-59` |
| Tracing setup | `backend/src/main.rs:35-42` |
| Graceful shutdown | `backend/src/main.rs:44-67` |
| Docker dep-cache + touch trick | `backend/Dockerfile:7-18` |
| Compose services | `docker-compose.yml:3-69` |
| nginx API proxy | `frontend/nginx.conf:13-27` |
| Frontend API client | `frontend/src/api.ts:17-59` |
| Frontend response types | `frontend/src/types.ts:1-50` |
| Portal tabs | `frontend/src/App.tsx:14-53` |

---

## Integration tests (`backend/tests/api.rs`)

| Test | Location |
|---|---|
| `health_is_always_ok` | `:42` |
| `readiness_reports_disabled_dependencies` | `:49` |
| `redact_endpoint_masks_all_categories` | `:58` |
| `redact_rejects_non_json_body` | `:81` |
| `redact_requires_text_field` | `:93` |
| `redact_accepts_empty_text` | `:99` |
| `deadline_shifts_across_weekend` | `:107` |
| `deadline_shifts_across_holiday` | `:122` |
| `deadline_supports_business_day_counting` | `:135` |
| `deadline_rejects_bad_date_and_state` | `:153` |
| `compliance_validates_tax_id_and_register_number` | `:182` |
| `compliance_reports_specific_errors` | `:197` |
| `compliance_requires_at_least_one_field` | `:211` |
| `holidays_endpoint_lists_state_specific_days` | `:218` |
| `responses_carry_request_id_header` | `:245` |
| `unknown_route_is_404` | `:254` |

Run everything:

```bash
cd backend && cargo test
```

---

## Endpoint quick reference

| Method | Path | Handler |
|---|---|---|
| GET | `/health` | `routes/health.rs:12` |
| GET | `/ready` | `routes/health.rs:18` |
| POST | `/api/v1/redact` | `routes/redact.rs:29` |
| POST | `/api/v1/deadline` | `routes/deadline.rs:48` |
| GET | `/api/v1/holidays?year=&state=` | `routes/deadline.rs:130` |
| POST | `/api/v1/compliance/validate` | `routes/compliance.rs:30` |

Live request sequence: repository root `api.http`.
