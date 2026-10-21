# Design Inputs — filled in for this repository

This is the "system" described in `lesson-plans.txt`. Every session, worksheet,
and assessment in this package is derived from the nine inputs below. Change an
input here and the downstream artifacts should be revised to match.

---

## 1. Learning goal

By the end of the course, learners can **design and justify the logic of a small
GovTech microservice**, specifically:

1. Build a correct, single-pass text redactor and reason about false positives
   vs. false negatives in pattern matching.
2. Translate a legal rule ("a deadline ending on a weekend/holiday shifts to the
   next business day") into tested code, including computing moving holidays.
3. Implement and validate a real check-digit algorithm (German VAT ID, mod-11).
4. Explain how a typed service layer is exposed as an HTTP API and how
   cross-cutting concerns (errors, rate limiting, caching, audit, health) are
   layered around it.

**Success looks like:** a learner can take a requirements paragraph in the style
of `challenge.md`, decompose it into pure functions, write tests for the edge
cases (empty input, year-end, leap day, invalid check digit), and wire the
functions into an endpoint — matching the quality bar of this repo.

---

## 2. Learner profile

- **Primary audience:** software developers (backend or full-stack), 1+ years of
  professional or upper-undergraduate experience.
- **Class size:** 8–24 (works for solo self-study too).
- **Languages they likely know:** Python, Node.js, Java, or C#.
- **Rust experience:** mixed — some none. The course never assumes Rust; Rust is
  introduced just-in-time as "the language this repo happens to use".
- **Domain experience:** little to none in German administrative law or DSGVO.
  All legal vocabulary is defined in `references/glossary.md`.
- **Motivation:** often preparing for a GovTech/LegalTech role or a coding
  interview; value practical, realistic problems over abstract algorithms.

---

## 3. Prior knowledge

**Assumed (build on it):**
- Writing and calling functions; using arrays/lists and dictionaries/maps.
- Reading and writing JSON.
- Basic command line; running a test command.
- The idea of a unit test (`assert expected == actual`).

**Not assumed (teach or scaffold explicitly):**
- Regular expressions beyond the basics.
- Rust syntax, ownership, or the `regex`/`chrono` crates.
- HTTP method/status semantics (400 vs 422 vs 429).
- Date arithmetic across time zones and years.
- Docker or container builds.

**Common misconceptions to pre-empt:**
- "A bigger regex catches more, so it's safer." (It also creates false
  positives — the source of the real bug in this repo.)
- "A holiday list is a static lookup." (Easter-based holidays move every year.)
- "`DE` + 9 digits is enough to validate a VAT ID." (There is a check digit.)

---

## 4. Lesson sequence

Four 90-minute sessions, each following the same rhythm
(**Hook → Concept → Code walk → Lab → Debrief**):

| # | Session | Core files | Capstone link |
|---|---|---|---|
| 1 | Redaction automaton (GDPR) | `services/redaction.rs` | Task 1 |
| 2 | Deadlines & holidays | `services/deadline.rs`, `services/holidays.rs` | Task 2 |
| 3 | Compliance & check digits | `services/compliance.rs` | Task 3 |
| 4 | Full-stack integration & shipping | `routes/`, `error.rs`, `middleware/`, `repository/`, `Dockerfile`, `frontend/` | wrap-up |

The sequence is ordered by dependency: Session 2 uses the "pure function +
tests" habit from Session 1; Session 4 assumes all three pure services exist.
Sessions 1–3 are independent enough to teach in any order if needed.

---

## 5. Learning activities

Each session mixes four activity types so no learner is passive for long:

- **Predict-then-run (whole class):** before running a test or `api.http`
  request, learners predict the output. Reveals the edge cases.
- **Code walk (guided):** the teacher navigates the real file, pointing at the
  exact lines in `references/code-map.md`.
- **Hands-on lab (pairs):** learners extend or break-and-fix the code
  (`worksheets/lab-*.md`). Includes a deliberately wrong starter so they must
  diagnose, not just transcribe.
- **Debrief / exit ticket (individual):** a one-paragraph "what rule governs
  this?" answer checked against the quiz.

---

## 6. Assessment evidence

Learning is evidenced through four instruments (all in `assessments/`):

| Instrument | Weight | What it proves |
|---|---|---|
| Exit tickets (one per session) | 10% | Concept recall in the learner's own words. |
| Lab submissions (4) | 40% | Ability to extend/repair real code; checked with `rubric.md`. |
| Quiz (15 questions) | 20% | Coverage of all three domains + API semantics. |
| Capstone (`practical-challenge.md`) | 30% | End-to-end: requirements → working, tested feature. |

**Mastery threshold:** the capstone's "Correctness & edge cases" and "Testing"
rows must reach *Proficient*; everything else at least *Developing*.

---

## 7. Output requirements

For every generated artifact in this package:

- **Format:** Markdown, black-on-white-friendly, no reliance on color alone.
- **Code references:** always `path/file.ext:line` so it is clickable/greppable.
- **Commands:** copy-pasteable, runnable from the stated directory.
- **Length:** session plans ≤ 2 pages equivalent; worksheets ≤ 1 page.
- **Language:** English prose; keep German domain terms and code identifiers
  verbatim (they are the thing being taught).
- **No answers in student-facing worksheets;** answer keys live only in
  `assessments/` or clearly marked "Teacher notes" blocks.

---

## 8. Accessibility & supports

- **Multiple representations:** every logic rule is given as (a) prose,
  (b) a worked example with dates/inputs, and (c) the code line.
- **Scaffolds:** each lab has a "Hint" ladder (nudge → stronger hint → reference
  line). Starter code snippets are provided for learners new to Rust.
- **Extensions:** every lab has a "Stretch" task for fast finishers (e.g.,
  `Av`-prefixed numbers, time zones, additional states).
- **Language:** German terms always glossed in `glossary.md`; no activity
  requires reading German prose.
- **Tooling:** labs run with a single command and need no database for Sessions
  1–3, so no learner is blocked by infrastructure.
- **Pacing:** each session has explicit timings and a "if behind, cut this"
  marker for the teacher.

---

## 9. Teacher decisions (defaults chosen for you)

These are opinionated defaults so the package works out of the box. Override any
of them and note the change here:

- **Language of instruction:** English (German domain terms kept verbatim).
- **Rust vs. language-agnostic:** teach in Rust because the reference
  implementation is Rust, but every concept is presented so it transfers to
  Python/Node/Java; a "port to your language" stretch task is offered.
- **Testing-first stance:** tests are written/predicted before the code walk, to
  model the repo's own discipline (`cargo test`, `-D warnings`).
- **No DB required in Sessions 1–3:** keep the barrier to entry low; Postgres and
  Redis only appear in Session 4.
- **Time-boxing is sacred:** the capstone mirrors the original 90-minute limit;
  "pragmatism over completeness" is an explicit, rewarded value (see
  `rubric.md`).
- **Assessment philosophy:** prefer evidencing understanding (predict, explain)
  over memorization (recall definitions).
