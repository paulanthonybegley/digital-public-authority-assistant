# Lesson Plans — Building the Digital Public Authority Assistant

A ready-to-teach course package built from the actual code in this repository.
Every concept taught here maps to a real file, function, or test in
`backend/`, `frontend/`, and `docker-compose.yml`.

The package is a **system**, not a single prompt: each lesson supplies the
learning goal, sequence, assessment evidence, and supports so you can deliver
it directly or adapt it to your context.

---

## What this package contains

| Path | Purpose |
|---|---|
| `00-design-inputs.md` | The nine design inputs (goal, learners, prior knowledge, sequence, activities, assessment, outputs, accessibility, teacher decisions) filled in for this repo. **Read this first.** |
| `sessions/session-1-redaction.md` … `session-4-full-stack.md` | Four 90-minute lesson plans, one per major idea. |
| `worksheets/lab-1-redaction.md` … `lab-4-integration.md` | Student-facing lab sheets handed out in each session. |
| `assessments/quiz.md` + `quiz-answer-key.md` | 15-question formative/summative quiz. |
| `assessments/practical-challenge.md` | A 90-minute capstone that mirrors the original hiring challenge. |
| `assessments/rubric.md` | Scoring rubric used for the capstone and the labs. |
| `references/architecture.md` | System diagram and request lifecycle. |
| `references/code-map.md` | Concept → `file:line` index so you can jump straight to the code. |
| `references/glossary.md` | Domain + technical vocabulary (DSGVO, VwVfG, mod-11, computus, …). |

---

## The four ideas being taught

1. **GDPR redaction as a single-pass automaton** — one combined regex, ordering
   rules, and the false-positive/false-negative tradeoff
   (`backend/src/services/redaction.rs`).
2. **Statutory deadline calculation** — calendar vs business-day counting, the
   weekend/holiday roll-forward rule, and computing Easter with the anonymous
   Gregorian computus instead of hardcoding dates
   (`backend/src/services/deadline.rs`, `holidays.rs`).
3. **Master-data compliance** — normalization + the official mod-11 product
   method for German VAT IDs, and structural register-number validation
   (`backend/src/services/compliance.rs`).
4. **Shipping it as a system** — typed errors → HTTP, layered middleware,
   Postgres audit without PII, Redis caching/rate-limiting that fails open,
   React client, and container builds (`routes/`, `middleware/`, `repository/`,
   `frontend/`, `Dockerfile`, `docker-compose.yml`).

---

## Suggested delivery formats

| Format | Use |
|---|---|
| **Full course** — 4 × 90 min + capstone | Sessions 1–4 then `practical-challenge.md`. |
| **Single 90-min workshop** | Session 1 (redaction) core + Lab 1 + a demo of Sessions 2–3. |
| **Self-paced / flipped** | Assign `references/*` + worksheets as reading; run only the coding labs. |
| **Interview prep** | `practical-challenge.md` + `rubric.md`; uses `challenge.md` as the brief. |

---

## How to run the code the lessons use

Students work against the same commands the project uses:

```bash
# Backend: unit + integration tests (no Postgres/Redis needed)
cd backend
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Frontend: component/client tests + typecheck + build
cd frontend
npm ci
npm test
npm run typecheck
```

Full stack (used from Session 4 onward):

```bash
docker compose up -d
# Web portal:  http://localhost:3000
# API health:  http://localhost:8080/health
```

The repository also ships `api.http` at its root: an ordered request sequence
that exercises every endpoint. Sessions 1–3 can be taught against it without
writing a line of code.

---

## The design-input template

This package was generated from `lesson-plans.txt`, which specifies that a
lesson plan must supply: learning goal, lesson sequence, assessment evidence,
learner profile, prior knowledge, learning activities, output requirements,
accessibility & supports, and teacher decisions. `00-design-inputs.md` answers
all nine for this repository; the session files then expand them into
minute-by-minute plans.

To retarget the package (different age group, language, or duration), edit
`00-design-inputs.md` first — the sessions inherit from it.

---

## Maintaining this package

Every `file:line` reference in these documents was verified against the repo
with `.verify_refs.py`:

```bash
python3 lesson-plans/.verify_refs.py
```

It resolves the shorthand convention (bare `redaction.rs:12` means
`backend/src/services/redaction.rs`; `Dockerfile:7` means `backend/Dockerfile`)
and checks that the line exists and that named test-table rows match the actual
test function. Re-run it after moving code.

