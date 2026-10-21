# Session 3 — Compliance: VAT IDs & Register Numbers

**Duration:** 90 minutes  **Primary code:** `backend/src/services/compliance.rs`
**Capstone link:** Task 3 of `challenge.md`  **Lab:** `worksheets/lab-3-compliance.md`

## Learning goal (this session)

Learners can implement the German VAT-ID **mod-11 product method** check digit,
separate **normalization** from **validation**, and choose the strongest
*verifiable* rule when no check digit exists (register numbers).

## Success criteria

By the end, learners can:
1. Normalize `" de123.456.78 8 "` → `DE123456788` and validate it as *valid*.
2. Compute the check digit for `12345678` by hand and get `8`.
3. Explain why `DE111111110` is invalid even though it "looks" fine.

## Prior knowledge to activate (3 min)

Ask: how does a website know you mistyped a credit-card number? (Check digit /
Luhn.) Establish the idea: a redundant digit detects most typos. German VAT IDs
use a mod-11 variant.

---

## Sequence

### 0:00–0:08 — Hook: input you cannot trust

Show a form where a company typed `de123.456.78 8` and `hrb12345b`. Ask: "Is
this the company's fault, or the validator's?" Establish the design principle:
**normalize first, then validate** — be liberal in what you accept, strict in
what you verify.

### 0:08–0:18 — Concept: normalize vs. validate

From `compliance.rs`:
- `normalize_tax_id` (`compliance.rs:36-42`): trim, uppercase, strip
  `space . - /`.
- `validate_tax_id` (`compliance.rs:66-96`): empty → `Empty`;
  not `DE`+9 digits → `InvalidFormat`; else compute the check digit and compare.
- The typed error enum `TaxIdError { Empty, InvalidFormat, InvalidCheckDigit }`
  (`compliance.rs:8-24`) serialized as `snake_case` — clients get a *machine*
  reason, not just `false`.

Point out the return shape `TaxIdReport { valid, normalized, error,
error_message }` (`compliance.rs:26-33`): it reports *both* the verdict and the
cleaned value, so the UI can display "we normalized your input to X".

### 0:18–0:36 — Concept: the mod-11 product method

Walk `compute_check_digit` (`compliance.rs:47-63`) step by step on the board.
Weights for the first 8 digits: **`2,7,6,5,4,3,2,1`**.

Worked example — `12345678`:

| digit | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| weight | 2 | 7 | 6 | 5 | 4 | 3 | 2 | 1 |
| product | 2 | 14 | 18 | 20 | 20 | 18 | 14 | 8 |

Digit-sum each product (e.g. `14 → 1+4 = 5`, `18 → 9`):

```
2 + 5 + 9 + 2 + 2 + 9 + 5 + 8 = 42
check = (10 - 42 % 10) % 10 = (10 - 2) % 10 = 8
```

Hence `DE12345678` **`8`** is valid (`known_valid_vat_id`, `compliance.rs:206`).
Note the code computes the digit-sum with `(product % 10) + (product / 10)`
(`compliance.rs:57-58`).

**The `0` rule:** the check digit must never be `0` (`compliance.rs:62`). So
`11111111` → sum 30 → check 0 → rejected. This is `check_digit_zero_is_invalid_by_rule`
(`compliance.rs:214`). This is the "looks fine" trap.

Then the "wrong digit" case: `DE123456789` has check digit `9 ≠ 8` →
`InvalidCheckDigit` (`wrong_check_digit_rejected`, `compliance.rs:222`).

### 0:36–0:46 — Concept: register numbers have no check digit

`validate_register_number` (`compliance.rs:131-199`). Read the doc comment
(`compliance.rs:125-130`): the register assigns sequential numbers, so there is
**nothing to compute**. The strongest verifiable rule is structural:
- accepted prefixes `HRB, HRA, HR, PR, GnR, VR` (`compliance.rs:158-165`);
- 2–6 digits;
- optional single trailing letter (`compliance.rs:179-184`);
- canonical display form (`GnR 88`, not `GNR 88`) via the prefix map.

This is a lesson in *epistemic honesty*: when you cannot verify, validate only
what you truly can. The `fuzz_survives_arbitrary_input` test (`compliance.rs:255`)
shows the contract — never panic, always report.

### 0:46–0:52 — Break

### 0:52–1:22 — Lab 3 (pairs)

`worksheets/lab-3-compliance.md`:
1. Hand-compute check digits for two IDs; confirm with the code.
2. Add a **unit test** for a new normalized input (`"DE-123-456-788"`).
3. Diagnose why `"DE000000000"` is rejected and which error is returned.
4. Stretch: sketch what would change to support `ATU`/`FR` prefixes.

### 1:22–1:30 — Debrief + exit ticket

Exit ticket: "Normalization and validation are different because… The check
digit for `12345678` is 8 because… Register numbers have no check digit, so the
strongest rule is…"

## If you are behind (cut here)

Cut the register-number section to a 3-minute mention; the mod-11 arithmetic and
the `0` rule are the core transferable skills.

## Materials

- `backend/src/services/compliance.rs`
- `api.http` (the `4.x` compliance block) for live demo
- Worksheet: `worksheets/lab-3-compliance.md`

## Teacher notes

- Do the mod-11 arithmetic by hand at least once; learners remember the
  weighting far better after doing it on paper.
- The `0`-is-forbidden rule is the single best "gotcha" question for the quiz.
- If time allows, show that normalization is why the endpoint is forgiving while
  still being strict about the true check.
