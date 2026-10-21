# Lab 2 — Statutory Deadlines & Holidays

**Time:** ~30 minutes  **Work in pairs**  **No database required**

**Reference code:** `backend/src/services/deadline.rs`,
`backend/src/services/holidays.rs`

```bash
cd backend
cargo test --lib deadline
cargo test --lib holidays
```

---

## Part A — Predict, then run

The rule under test (VwvVfG § 32 / ZPO § 193): *if a period ends on a Saturday,
Sunday, or public holiday, it ends on the next business day.*

Fill in the **result date**, **shifted?**, and **reason** before running.

| # | start_date | days | state | counting | result | shifted | reason |
|---|---|---|---|---|---|---|---|
| A1 | 2026-01-05 (Mon) | 3 | BE | calendar | | | |
| A2 | 2026-01-02 (Fri) | 1 | BE | calendar | | | |
| A3 | 2026-04-30 (Thu) | 1 | BE | calendar | | | |
| A4 | 2025-12-31 (Wed) | 1 | BE | calendar | | | |
| A5 | 2026-04-30 (Thu) | 1 | BE | business | | | |

Check yourself by calling the service from a scratch test, or post to the API if
you have it running (see `api.http`, block `2.x`).

**Question A6:** A3 and A5 return the **same date** (`2026-05-04`) but different
`shifted` values. Explain why.

> Hint: look at `deadline.rs:107-151`.

---

## Part B — Break and understand

Find `consecutive_holiday_weekend_bridge` (`deadline.rs:196`). It tests
`2026-10-30 + 1` in **BY** and expects `2026-11-02`.

1. Temporarily change the Corpus Christi offset from `+60` to `+61` and run the
   tests. Which test(s) fail?
2. Revert it. Now explain why `2026-10-31` (Reformationstag) plus the following
   Sunday forces the result to Monday `2026-11-02` **in BY but not in BE**.
   (Hint: check which states observe Reformationstag at `holidays.rs:152-158`.)

Restore the file exactly (the offset must be `+60`).

---

## Part C — Easter without hardcoding

1. Run `cargo test --lib holidays::tests::easter_known_years` and note the four
   dates.
2. **Predict** Easter Sunday for 2027 without running the code. Then check it.
3. In one sentence, explain why hardcoding moving holidays is a maintenance
   hazard (look at `holidays.rs:1-4` and `:130-134`).

**Question C4:** Which Easter-derived holidays does the code add, and at which
day offsets?

> Hint: `holidays.rs:131-134` and `:150`.

---

## Part D — Add a guard

Add a test that proves a deadline landing on **New Year's Day** rolls forward:

- pick a start date such that `start + days` is exactly `2026-01-01` (a holiday),
- assert the result is `2026-01-02` and the reason is `PublicHoliday`.

**Acceptance:** `cargo test --lib deadline` passes with your new test.

---

## Part E — Stretch (fast finishers)

- The range guard is `1900..=2100` (`deadline.rs:84`). Why is an explicit range
  guard useful even though `NaiveDate` already enforces valid calendar dates?
- `parse_date` rejects `2026-1-5` and `05.01.2026`. Find the assertion list in
  `rejects_invalid_dates` (`deadline.rs:266`) and add one more malformed shape
  you can think of, then confirm it is rejected.

---

## Submit

- Part A table + A6 answer.
- Part B written explanation (file restored).
- Part C predictions + C4 answer.
- The new New-Year test (Part D).

## Hint ladder (only if stuck)

1. *Nudge:* calendar counting adds days first and rolls the *result*; business
   counting only counts up on business days.
2. *Stronger:* a business day = not Sat/Sun **and** not in `public_holidays`.
3. *Reference:* `deadline.rs:96-159`, `holidays.rs:106-183`.
