# Lab 3 — Compliance: VAT IDs & Register Numbers

**Time:** ~30 minutes  **Work in pairs**  **No database required**

**Reference code:** `backend/src/services/compliance.rs`

```bash
cd backend
cargo test --lib compliance
```

---

## Part A — Hand-compute the check digit

The official method (product method, § 150 AO): weight the first 8 digits with
`2,7,6,5,4,3,2,1`, take the **digit sum** of each product, sum them, then
`check = (10 - sum % 10) % 10`. A check digit of `0` is **never** allowed.

Complete the table for `12345678`:

| digit | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| weight | 2 | 7 | 6 | 5 | 4 | 3 | 2 | 1 |
| product | 2 | 14 | 18 | 20 | 20 | 18 | 14 | 8 |
| digit sum | 2 | | 9 | | | 9 | 5 | 8 |

- Total sum = ____
- Check digit = `(10 - total % 10) % 10` = ____
- Is `DE12345678` + that digit valid? Check with the code.

**Question A1:** Why is `DE123456789` **invalid** even though it has the right
shape and a nonzero check digit?

**Question A2:** Compute the check digit for `11111111`. What is it, and why is
the result nevertheless rejected?

> Hint: `compliance.rs:47-63` and the tests at `:213` and `:221`.

---

## Part B — Normalization vs. validation

1. Show, in your own words, what `normalize_tax_id` does to
   `" de123.456.78 8 "` (`compliance.rs:36-42`).
2. Add a **new test** proving that `"DE-123-456-788"` normalizes to
   `DE123456788` and is valid.
3. Run `cargo test --lib compliance`.

**Question B4:** The endpoint reports both `valid` and `normalized`. Why might a
UI want the normalized value even when the input was *invalid*?

> Hint: `TaxIdReport` at `compliance.rs:26-33`.

---

## Part C — Register numbers: validate what you can

1. Read the doc comment above `validate_register_number`
   (`compliance.rs:125-130`). Why is there **no** check digit to verify?
2. List the accepted prefixes (`compliance.rs:158-165`).
3. For each input, predict valid/invalid **before** running:

| Input | Valid? | Normalized (if any) |
|---|---|---|
| `HRB 12345 B` | | |
| `hrb12345b` | | |
| `GnR 88` | | |
| `HRB 1 B` | | |
| `HRB 1234567 B` | | |
| `XYZ 12345` | | |

**Question C4:** `GnR 88` is normalized to `GnR 88`, not `GNR 88`. Where is the
display form defined, and why normalize to a canonical form at all?

> Hint: `compliance.rs:157-172`.

---

## Part D — Robustness contract

Run `cargo test --lib compliance::tests::fuzz_survives_arbitrary_input`.

**Question D1:** What is the **contract** this test enforces? (It is not about a
specific answer.)

**Task D2:** Add one more hostile input to that test (emoji, null byte, very long
string) and confirm the function reports instead of crashing.

---

## Part E — Stretch (fast finishers)

Sketch (in prose, no code needed) how you would extend the VAT check to accept
other EU prefixes (`ATU`, `FR`, `NL`). What is the *minimum* that can be verified
per country, and what would you deliberately refuse to verify offline?

---

## Submit

- Hand-computed table + A1, A2 answers.
- The `DE-123-456-788` test (Part B) passing.
- Part C table + C4.
- D1 answer and the extended fuzz input.

## Hint ladder (only if stuck)

1. *Nudge:* normalization removes ` `, `.`, `-`, `/` and uppercases.
2. *Stronger:* the "digit sum" of 14 is 1+4; the code does `(p % 10) + (p / 10)`.
3. *Reference:* `compliance.rs:47-96` (VAT), `:131-199` (register).
