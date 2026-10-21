# Lab 4 — From Pure Functions to a Shipped System

**Time:** ~30 minutes  **Work in pairs**
**Requires:** Docker (or a locally running backend). This is the only lab that
uses infrastructure.

**Reference code:** `backend/src/lib.rs`, `error.rs`, `state.rs`,
`routes/*`, `middleware/rate_limit.rs`, `repository/audit.rs`,
`backend/Dockerfile`, `docker-compose.yml`, `api.http`

Start the stack:

```bash
docker compose up -d
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

---

## Part A — Read the layer onion

Open `build_router` (`lib.rs:28-42`).

1. List the layers from **inner** (closest to handlers) to **outer**.
2. For each layer, write a one-line "why does this exist".

**Question A3:** `SetRequestIdLayer` is the outermost and
`PropagateRequestIdLayer` is just inside it. Explain the difference: which one
*creates* an id, and which one *copies it onto the response*?

> Hint: `lib.rs:40-41`; the test `responses_carry_request_id_header` is at
> `tests/api.rs:245`.

---

## Part B — Errors map to status codes

Open `error.rs`. Fill in the table:

| Trigger | `AppError` variant | HTTP status |
|---|---|---|
| body is not JSON | | |
| unknown federal state `XX` | | |
| `days_to_add: -3` | | |
| more than N requests/minute | | |
| Postgres unreachable at startup | | |

Verify three of them live:

```bash
# invalid state -> expect 422
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://localhost:8080/api/v1/deadline \
  -H 'content-type: application/json' \
  -d '{"start_date":"2025-12-31","days_to_add":1,"federal_state":"XX"}'

# bad date -> expect 400
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://localhost:8080/api/v1/deadline \
  -H 'content-type: application/json' \
  -d '{"start_date":"31.12.2025","days_to_add":1,"federal_state":"BE"}'

# missing fields -> expect 400
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://localhost:8080/api/v1/compliance/validate \
  -H 'content-type: application/json' -d '{}'
```

---

## Part C — Observe caching and audit

1. Call the deadline endpoint once, note `"cached"`, then call it again with the
   **same** parameters. What changes?
   ```bash
   curl -s -X POST http://localhost:8080/api/v1/deadline -H 'content-type: application/json' \
     -d '{"start_date":"2026-04-30","days_to_add":1,"federal_state":"BE","counting":"calendar"}'
   ```
2. Call redact and note the `audit_id`:
   ```bash
   curl -s -X POST http://localhost:8080/api/v1/redact -H 'content-type: application/json' \
     -d '{"text":"max@example.de, DE89370400440532013000, 030/1234567"}'
   ```
3. Inspect the audit row (metadata only — confirm there is no text column):
   ```bash
   docker compose exec postgres psql -U authority -d authority \
     -c 'SELECT id, payload_sha256, email_count, iban_count, phone_count FROM redaction_audit;'
   ```

**Question C4:** The redaction still succeeds even if the audit insert fails
(`routes/redact.rs:51-68`). Argue for and against that choice.

**Question C5:** The audit stores `payload_sha256` and counts, but no text. Explain
how this still satisfies an audit requirement while respecting data minimization.

---

## Part D — Failing open

Read `middleware/rate_limit.rs:76-79` and `routes/deadline.rs:75-92`.

1. State in one sentence what "fail open" means here.
2. Describe a scenario where failing open would be **dangerous**.
3. Describe a scenario where failing open is clearly **correct** (this system).

**Question D3:** `AppState::for_tests` (`state.rs:64-70`) sets `db` and `redis`
to `None`. Why is that the reason the integration tests (`tests/api.rs`) need no
Docker at all?

---

## Part E — DevOps case study

Read `backend/Dockerfile:1-18`. The image once shipped a **332 KB binary that
did nothing** (an empty `fn main() {}`).

1. Explain the dependency-cache trick (`Dockerfile:7-11`).
2. Explain why cargo thought the stub was "up to date" after the real sources
   were copied in.
3. Explain what `find src -name '*.rs' -exec touch {} +` (`Dockerfile:17`) does
   and why it fixes the problem.

**Question E4:** Without the `touch`, what would a user of the API observe, and
why would it be confusing to debug?

---

## Part F — Stretch (fast finishers)

Trigger the rate limiter (limit is 120/min; see `RATE_LIMIT_PER_MINUTE` in
`docker-compose.yml`) and inspect the `retry-after`, `x-ratelimit-limit`, and
`x-ratelimit-remaining` headers (`rate_limit.rs:52-73`). Describe the
fixed-window algorithm and one of its weaknesses (the "boundary" problem).

---

## Submit

- Part A layer list + A3.
- Part B table + the three observed status codes.
- Part C observations + C4, C5.
- Part D answers.
- Part E explanation.

## Hint ladder (only if stuck)

1. *Nudge:* layers wrap the router like an onion; the outermost sees the request
   first and the response last.
2. *Stronger:* `for_tests` disables backing services, so handlers take their
   "no persistence" branch.
3. *Reference:* `references/architecture.md` for the full request lifecycle.
