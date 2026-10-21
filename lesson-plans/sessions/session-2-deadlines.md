# Session 2 — Statutory Deadlines & Public Holidays

**Duration:** 90 minutes
**Primary code:** `backend/src/services/deadline.rs`, `backend/src/services/holidays.rs`
**Capstone link:** Task 2 of `challenge.md`  **Lab:** `worksheets/lab-2-deadlines.md`

## Learning goal (this session)

Learners can translate a legal rule into tested code, distinguish **calendar-day**
from **business-day** counting, and compute a *moving* holiday (Easter) instead of
hardcoding it.

## Success criteria

By the end, learners can:
1. State the rule: "a period ending on a Saturday, Sunday, or public holiday
   ends on the next business day."
2. Trace `calculate_deadline("2026-04-30", 1, BE, calendar)` to `2026-05-04` and
   name the two reasons it moved.
3. Explain why the holiday list must be computed, not stored.

## Prior knowledge to activate (3 min)

Ask: "What date is Easter in 2027? 2030?" (They won't know.) Draw out that
Easter *moves* each year and that several German holidays are defined relative
to it. This motivates the computus.

---

## Sequence

### 0:00–0:08 — Hook: the missed deadline

Scenario: an authority must issue a decision within **2 weeks** of an
application. The naive `start + 14 days` lands on a Saturday. Which date goes in
the letter? Show the consequence of getting it wrong (an appealable error).
Board the rule, cite the intent of **VwVfG § 32 / ZPO § 193** (glossed in
`references/glossary.md`).

### 0:08–0:20 — Concept: two ways to count

From `DayCounting` (`deadline.rs:10-36`):
- `Calendar`: add N calendar days, then **roll forward** if the result is not a
  business day (`deadline.rs:108-118`).
- `Business`: step one day at a time and only decrement the counter on business
  days (`deadline.rs:119-134`).

Contrast with the same input `2026-04-30 + 1`:
- calendar → raw `2026-05-01` (Labour Day) → roll to `2026-05-04`.
- business → skips Fri/Sat/Sun → `2026-05-04`, and `shifted == false`.

This is the subtle part: **same date, different meaning of "shifted."** In
calendar mode the result moved (`shifted = true`); in business mode the result
was always a business day (`shifted = false`). Point at the branch at
`deadline.rs:137-151`.

### 0:20–0:34 — Code walk: holidays as computation

Open `holidays.rs`.
- `FederalState` enum + case-insensitive `parse` (`holidays.rs:10-54`).
- **Easter computus**, the anonymous Gregorian algorithm (`holidays.rs:78-94`).
  Walk it as arithmetic, not magic: the `a…m` steps produce month/day.
  Verify with the test `easter_known_years` (`holidays.rs:190`): 2024-03-31,
  2026-04-05, 2000-04-23.
- Holidays derived from Easter by offsets:
  Good Friday `-2`, Easter Monday `+1`, Ascension `+39`, Pentecost Monday `+50`,
  Corpus Christi `+60` (`holidays.rs:131-134`, `:150`).
- `repentance_day` — Wednesday strictly before 23 Nov, SN only
  (`holidays.rs:97-103`).
- `is_business_day` = not weekend AND not holiday (`holidays.rs:181-183`).

Stress the design value: **no hardcoded moving dates** — add a year and it just
works.

### 0:34–0:44 — Concept: strict parsing & typed errors

`parse_date` (`deadline.rs:69-88`) rejects single-digit forms that `chrono`
would accept leniently, plus a `1900..=2100` range guard. Show the rejected
list in `rejects_invalid_dates` (`deadline.rs:266`): `05.01.2026`, `2026-1-5`,
`20260105`, `2026-02-30`.

Then `DeadlineError` (`deadline.rs:38-46`) and its mapping to HTTP in
`error.rs:30-39`: invalid date → **400**, unknown state / negative days →
**422**. This is the seed of Session 4.

### 0:44–0:50 — Break

### 0:50–1:22 — Lab 2 (pairs)

`worksheets/lab-2-deadlines.md`:
1. Predict-then-run the four scenarios in the quiz table (weekend, holiday,
   year-end, business mode).
2. Break a test on purpose: change an offset and watch `consecutive_holiday_weekend_bridge`
   (`deadline.rs:196`) fail, then reason about the chain.
3. Extend: add a check that a deadline on **New Year's Day** rolls to Jan 2.

### 1:22–1:30 — Debrief + exit ticket

Exit ticket: "Calendar-day counting differs from business-day counting in that…
`shifted` can be false even when the raw date moved because… I would not
hardcode Easter because…"

## If you are behind (cut here)

Cut the computus arithmetic walk to a 2-minute overview (it is provided
complete in code) and keep the two counting modes and the error mapping.

## Materials

- `backend/src/services/deadline.rs`, `backend/src/services/holidays.rs`
- `api.http` (the `2.x` and `3.x` blocks) for live demo
- Worksheet: `worksheets/lab-2-deadlines.md`

## Teacher notes

- The "shifted=false in business mode" idea is the most commonly missed point —
  make learners write it in their own words.
- If learners know a language with a `LocalDate` type, invite the cross-language
  comparison; it reinforces that the algorithm is portable.
- Keep the legal citations light: the *rule* is what matters, not the paragraph
  numbers.
