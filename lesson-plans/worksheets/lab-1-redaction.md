# Lab 1 — The Redaction Automaton (GDPR Masking)

**Time:** ~30 minutes  **Work in pairs**  **No database required**

**Reference code:** `backend/src/services/redaction.rs`

Run the tests at any time:

```bash
cd backend
cargo test --lib redaction
```

You will see all tests pass. Your job is to *understand, predict, and extend* —
not to make the existing tests pass.

---

## Part A — Predict, then run

For each input below, **write your predicted redacted output** before running
anything. A quick way to run a single case is to add a temporary test, or use
this one-liner (paste your text):

```bash
cargo test --lib redaction -- --nocapture
```

| # | Input | Your prediction |
|---|---|---|
| A1 | `Kontakt: max@example.de` | |
| A2 | `IBAN DE89370400440532013000` | |
| A3 | `Tel 030/1234567` | |
| A4 | `AZ-2026/0042 2026-05-01 1.234,56 EUR` | |
| A5 | `Mobil 0170 1234567` | |

**Question A6:** Input A4 contains numbers. Why is **nothing** redacted there?
Name the two rules in the phone pattern that prevent it.

> Hint: look at `redaction.rs:14-24`. One rule is about what a German national
> number may start with; the other is about how many digits the subscriber needs.

---

## Part B — Extend the tests

The tests live at the bottom of `redaction.rs`. Add **two** new tests:

1. `masks_number_that_must_be_masked` — pick a phone format that is *not*
   already in the test list and assert it is masked with `[PHONE_REDACTED]`.
2. `does_not_mask_that_should_survive` — pick a non-sensitive string containing
   digits (an order number, a price, a date) and assert `counts.total() == 0`.

**Acceptance:** `cargo test --lib redaction` still passes **and** your two tests
genuinely fail if you temporarily revert the fix described below.

---

## Part C — Diagnose the bug

Here is the *old, over-broad* national phone pattern:

```
\b0\d{2,4}[\s\-/]\d{3,}
```

1. Explain in one sentence why it matched `0042 2026` inside
   `AZ-2026/0042 2026-05-01`.
2. The fix changed it to:
   ```
   \b0[1-9]\d{0,3}[\s\-/]\d{5,}
   ```
   For **each** of the two changes (`0[1-9]` and `\d{5,}`), say which part of
   `0042 2026` it now rejects.

**Question C3:** Which is worse for a real authority: a **false positive**
(redacting text that was not sensitive) or a **false negative** (leaking real
PII)? Argue both sides in two sentences. Which did this repo choose to prioritize?

> Hint: the regression test is `does_not_mask_file_numbers_followed_by_dates`
> (`redaction.rs:184`) and the counter-test is
> `masks_short_but_valid_national_numbers` (`redaction.rs:205`).

---

## Part D — Stretch (fast finishers)

- The IBAN pattern is `DE\d{2}\s?(?:\d{4}\s?){4}\d{2}`. What class of *invalid*
  IBANs would still match it? (Check digits are not verified here.) Write down
  two examples of text that would match but is not a real IBAN.
- Propose (do not implement) how you would also mask **names**, given that names
  have no fixed pattern. What are the failure modes?

---

## Submit

- Your predicted table (Part A).
- The two new tests (Part B), passing.
- Your written answers to A6, C1, C2, C3.

## Hint ladder (only if stuck)

1. *Nudge:* the combined pattern is tried left-to-right; at each position the
   first alternative that matches wins.
2. *Stronger:* `\b` is a word boundary; `0[1-9]` means "a zero then a digit
   1–9".
3. *Reference:* `redaction.rs:11-27` (the pattern) and `redaction.rs:53-79`
   (the masking loop).
