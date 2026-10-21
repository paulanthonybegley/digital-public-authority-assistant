# Reference — Architecture

How the Digital Public Authority Assistant is put together. Use this as the
single diagram set for all four sessions.

---

## Container view

```
                    browser
                       │  http://localhost:3000
                       ▼
        ┌──────────────────────────────┐
        │  web  (nginx, frontend/)     │
        │  - serves React SPA          │
        │  - proxies /api, /health,    │
        │    /ready to api:8080        │
        └──────────────┬───────────────┘
                       │  docker network
                       ▼
        ┌──────────────────────────────┐        ┌──────────────┐
        │  api  (Rust/Axum, backend/)  │───────▶│ postgres:16  │
        │  port 8080                   │        │ audit trail  │
        │                              │        └──────────────┘
        │                              │───────▶┌──────────────┐
        │                              │        │  redis:7     │
        └──────────────────────────────┘        │ cache + rate │
                                                └──────────────┘
```

Source: `docker-compose.yml:3-69`; the API proxy rules are in
`frontend/nginx.conf:13-27`.

---

## Request lifecycle (backend)

`build_router` (`backend/src/lib.rs:28-42`) wraps the routes in layers. The
request travels **outermost → innermost**, and the response returns in reverse:

```
client
  │
  ▼  SetRequestIdLayer        (creates x-request-id if absent)      lib.rs:41
  ▼  PropagateRequestIdLayer  (copies the id onto the response)     lib.rs:40
  ▼  TraceLayer              (logs method, uri, latency, status)    lib.rs:39
  ▼  CorsLayer               (browser access policy)               lib.rs:38
  ▼  TimeoutLayer            (504 after 10s)                        lib.rs:34
  ▼  rate_limit middleware   (429 past the per-minute limit)        lib.rs:33
  ▼
routes::api_router          (routes/mod.rs:11-19)
  ├─ GET  /health              → health.rs:12
  ├─ GET  /ready               → health.rs:18
  ├─ POST /api/v1/redact       → redact.rs:29
  ├─ POST /api/v1/deadline     → deadline.rs:48
  ├─ GET  /api/v1/holidays     → deadline.rs:130
  └─ POST /api/v1/compliance/validate → compliance.rs:30
       │
       ▼  domain services (pure)          state.rs:11  (config, db?, redis?)
          ├─ redaction.rs  mask_sensitive_data        redaction.rs:53
          ├─ deadline.rs   calculate_deadline         deadline.rs:96
          │     └─ holidays.rs  is_business_day        holidays.rs:181
          └─ compliance.rs validate_tax_id            compliance.rs:66
                           validate_register_number   compliance.rs:131
```

### Cross-cutting concerns

| Concern | Where | Behaviour |
|---|---|---|
| Typed errors → HTTP | `error.rs:41-58` | 400 / 422 / 429 / 503 / 500 |
| Rate limit | `middleware/rate_limit.rs:14` | fixed window, **fails open** on Redis error (`:76-79`) |
| Deadline cache | `routes/deadline.rs:75-116` | Redis GET/SET, 3600s TTL, fails open |
| Audit trail | `repository/audit.rs:29` | metadata only: SHA-256 + counts |
| Config | `config.rs:38-59` | env vars with safe defaults |
| State | `state.rs:11-71` | `db`/`redis` optional → tests need no infra |

---

## Data flow: one redaction request

```
POST /api/v1/redact {"text": "...max@example.de..."}
   │
   ├─ rate_limit: INCR rate_limit:<client>            rate_limit.rs:40
   ├─ redact handler reads x-request-id               redact.rs:33
   ├─ parse JSON body (400 on failure)                redact.rs:39
   ├─ size guard > 1 MiB (400)                        redact.rs:43-47
   ├─ mask_sensitive_data(text)                       redaction.rs:53
   │     └─ single-pass combined regex                redaction.rs:11
   ├─ if db present: record_redaction(...)            redact.rs:54-68
   │     ├─ payload_digest = SHA-256(original)        audit.rs:24
   │     └─ INSERT counts + digest (no text)          audit.rs:37
   └─ 200 {"text": masked, "counts": {...}, "audit_id": ...}
```

The audit insert is best-effort: on error it logs and still returns 200
(`redact.rs:66`).

---

## Persistence

`migrations/0001_redaction_audit.sql` creates `redaction_audit`:

| column | notes |
|---|---|
| `id` UUID | primary key, returned as `audit_id` |
| `request_id` | ties the record to the `x-request-id` trace |
| `payload_sha256` | hex digest of the **pre-redaction** text (never the text) |
| `email_count`, `iban_count`, `phone_count` | per-category counts |
| `created_at` | timestamp, indexed descending for recent-first queries |

Migrations run automatically at startup (`state.rs:31-34`).

---

## Configuration

| Env var | Default | Meaning |
|---|---|---|
| `BIND_ADDR` | `0.0.0.0:8080` | listen address |
| `DATABASE_URL` | unset | Postgres; unset ⇒ audit disabled |
| `REDIS_URL` | unset | Redis; unset ⇒ cache + rate limit disabled |
| `RATE_LIMIT_PER_MINUTE` | `120` | per-client request budget |
| `CORS_ALLOW_ORIGIN` | `*` | allowed origin (`*` in dev) |
| `STRICT_READINESS` | `false` | if true, missing deps make `/ready` 503 |
| `RUST_LOG` | `info,tower_http=debug,public_authority_api=debug` | tracing filter |

Source: `config.rs:38-59`, `main.rs:35-42`, `docker-compose.yml:34-41`.

---

## Build & deploy

- **API image** (`backend/Dockerfile`): multi-stage. Build stage caches
  dependencies by compiling a throwaway `fn main(){}`, then rebuilds the real
  sources. `touch` (`Dockerfile:17`) defeats cargo's mtime-based staleness check
  so the stub is not reused. Runtime stage is `debian:bookworm-slim`, runs as a
  non-root user, exposes 8080.
- **Web image** (`frontend/Dockerfile`): Node build → nginx serve. nginx also
  reverse-proxies the API.
- **Compose** (`docker-compose.yml`): `postgres`, `redis`, `api`, `web` with
  health checks and `depends_on: condition: service_healthy`.

Health endpoints: `/health` is liveness (never touches deps, `health.rs:12`);
`/ready` checks Postgres + Redis and treats **disabled** (non-strict) as healthy
(`health.rs:74-85`).
