# Quiz — Answer Key & Marking Notes

**Teacher copy — do not distribute before the quiz.**

Total: 20 marks. Accept any answer that demonstrates the stated understanding;
the wording below is a model, not a script.

---

## Section 1 — Redaction

**1. [1]** A placeholder produced by one pass could be re-matched by a later
pass (e.g. the redacted output re-scanned as a phone number), corrupting the
result. *Any one concrete example scores the mark.*

**2. [1]** At a given position, the **first alternative that matches wins**
(leftmost alternative in the alternation). This is why order
`email | IBAN | phone` matters.

**3. [1]** `Kontakt: [EMAIL_REDACTED], Tel [PHONE_REDACTED]`

**4. [2]** (one mark each)
- National numbers never begin with `00` because that is the international
  prefix → rule `0[1-9]` rejects the `00` of `0042`.
- The subscriber part requires **≥ 5 digits** → `\d{5,}` rejects the 4-digit
  tail `2026`.

---

## Section 2 — Deadlines & holidays

**5. [2]** `2026-01-02` (Fri) + 1 day = Saturday `2026-01-03` → rolls to Monday
`2026-01-05`; `shifted = true`; `reason = weekend`.
*(1 mark date, 1 mark shifted+reason.)*

**6. [1]** In **business** counting, no non-business day is ever counted: the
algorithm only advances the counter on business days, so the result is a
business day *by construction*. `shifted` compares the raw calendar result with
the official date; here the official date is the only result produced, so it is
not considered shifted.

**7. [1]** Easter (and holidays defined relative to it) **moves every year**.
Computing it from the algorithm means any year works and there is no table to
maintain or update.

**8. [1]** A date is a business day if it is **not** a Saturday/Sunday **and**
**not** a public holiday in the given federal state.

---

## Section 3 — Compliance

**9. [2]** Products and digit sums:
`2, 5, 9, 2, 2, 9, 5, 8` → total **42** → `(10 - 42 % 10) % 10 = (10 - 2) % 10
= 8`. Check digit **8**. *(1 mark total, 1 mark digit.)*

**10. [1]** The check digit `0` is **never permitted** by the rule. The computed
check digit for `11111111` is 0, so the ID is rejected regardless of shape.

**11. [1]** **Normalization** (trim, uppercase, strip separators `space . - /`)
yields `DE123456788`. *("Normalization" or equivalent + correct result.)*

**12. [1]** The commercial register assigns **sequential** numbers without a
check digit, so there is nothing to compute; the strongest verifiable rule is a
structural/format check.

---

## Section 4 — System design

**13. [2]** (0.5 marks each)
- (a) invalid JSON → **400**
- (b) unknown state → **422**
- (c) rate limit exceeded → **429**
- (d) Postgres down at startup (readiness) → **503**

**14. [1]** **Data minimization** (GDPR Art. 5(1)(c)); the digest still lets you
prove that a specific input was processed (and detect repeated inputs) without
retaining the PII.

**15. [2]** "Fail open" means that when the dependency (Redis) is unavailable,
the system **continues to serve requests** rather than returning an error. It is
correct here because the primary job (redaction / deadline calculation) can still
be done — the cache and rate-limit are optimizations/protections, not the core
service — so blocking requests would harm availability for no privacy or
correctness benefit. *(Accept: "availability over audit".)*

---

## Marking bands

| Total | Band |
|---|---|
| 18–20 | Exceeds |
| 14–17 | Proficient |
| 10–13 | Developing |
| < 10 | Beginning — revisit the relevant session |
