# Capstone — Practical Coding Challenge

**Time:** 90 minutes  **Weight:** 30% of course grade  **Individual**
**Marked with:** `assessments/rubric.md`

This capstone mirrors the original hiring challenge in `challenge.md`. You may
work in **any language**; the reference implementation is Rust. You may not copy
from the repository's service files while writing — but you may read them
*afterwards* to compare. Your tests and design notes are part of the submission.

---

## Scenario

A state authority needs a small service that pre-processes incoming digital
filings. Build the three core components below. Correctness and edge cases
matter more than breadth; **pragmatism is rewarded** (clean mocks are fine).

Deliver either:
- **Option A (backend only):** three callable functions + tests.
- **Option B (full-stack):** the three functions exposed over HTTP + tests.

---

## Task 1 — Redaction automaton (GDPR)

Implement `mask_sensitive_data(text) -> string` that masks:
- email addresses → `[EMAIL_REDACTED]`
- German IBANs (`DE\d{2}\s?(\d{4}\s?){4}\d{2}`) → `[IBAN_REDACTED]`
- common German phone numbers (`+49 …`, `0049 …`, `030/…`, `0170 …`) →
  `[PHONE_REDACTED]`

**Required behaviours:**
- Non-sensitive text must be preserved **verbatim**, including umlauts.
- A date like `2026-05-01` or a file number like `AZ-2026/0042` must **not** be
  redacted.
- Adjacent entities must not interfere with each other.

**Tests you must write:** at least one true-positive per category, and at least
one true-negative (a date or file number that survives).

---

## Task 2 — Statutory deadline

Implement
`calculate_deadline(start_date, days_to_add, federal_state, counting) -> date`
where:
- `start_date` is strictly `YYYY-MM-DD`;
- `counting` is `calendar` (add days, then roll forward) or `business` (count
  only business days);
- a period ending on a Saturday, Sunday, or public holiday
  `federal_state` rolls to the next business day.

**Required behaviours:**
- At least **three** fixed national holidays (New Year, Labour Day, 3 October).
- Support at least **two** moving holidays, ideally computed (Easter-based)
  rather than hardcoded.
- Calendar vs. business counting must be distinguishable in the output.

**Tests you must write:** a weekday case, a weekend roll, a holiday roll, a
**year-end** transition, and an **invalid input** case.

---

## Task 3 — Master-data compliance

Implement `validate_tax_id(vat_id) -> result`:
- German VAT ID = `DE` + 9 digits;
- apply the **mod-11 product method** (weights `2,7,6,5,4,3,2,1`, digit-sum each
  product, `check = (10 - sum % 10) % 10`, and the check digit may **not** be 0);
- normalize input first (trim, uppercase, strip `space . - /`).

Optionally implement `validate_register_number` for `HRB/HRA/PR/GnR/VR/HR`
shapes.

**Tests you must write:** one valid ID, one wrong check digit, one bad format,
and the computed check digit for `12345678` (answer: 8).

---

## Submission requirements

1. **Source** for all three tasks.
2. **Test output** — paste the command you ran and its result.
3. **Design note (≤ 1 page):** for each task, state the key rule you encoded and
   one edge case you deliberately handled.
4. **Trade-off note (3–5 sentences):** in Task 1, where did you risk a false
   positive to avoid a false negative (or vice versa), and why?

---

## Timebox (recommended)

| Time | Activity |
|---|---|
| 0:00–0:05 | Read the brief; sketch data shapes. |
| 0:05–0:30 | Task 1 + its tests. |
| 0:30–0:58 | Task 2 + its tests. |
| 0:58–1:18 | Task 3 + its tests. |
| 1:18–1:25 | Paste test output; write the design note. |
| 1:25–1:30 | Self-check against `rubric.md`. |

## Comparison (only after time is up)

The reference implementation is:
- `backend/src/services/redaction.rs`
- `backend/src/services/deadline.rs` + `holidays.rs`
- `backend/src/services/compliance.rs`

Compare your design decisions. Differences are expected and welcome — note them
in your trade-off note.
