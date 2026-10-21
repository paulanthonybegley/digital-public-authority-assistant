# Quiz — Digital Public Authority Assistant

**15 questions · 20% of course grade · ~25 minutes**

Answer all questions. Show working for calculations. You may consult
`references/glossary.md` but not the source code. Marks in brackets.

---

## Section 1 — Redaction (5 marks)

**1.** [1] The redactor uses a *single combined* regex rather than three
sequential replacements. Give one concrete bug that sequential replacements would
introduce.

**2.** [1] In the combined pattern the alternatives are ordered
`email | IBAN | phone`. Explain the rule that decides which alternative matches
when two could match at the same position.

**3.** [1] Predict the output for:
`Kontakt: max@example.de, Tel 030/1234567`

**4.** [2] For the input `AZ-2026/0042 2026-05-01`, the redactor produces **no**
matches. Name the **two** rules in the phone pattern that prevent a false match,
and say which substring each rule rejects.

---

## Section 2 — Deadlines & holidays (5 marks)

**5.** [2] Compute the result of
`calculate_deadline("2026-01-02", 1, BE, calendar)`. State the date, whether
`shifted` is true, and the `reason`. (2026-01-02 is a Friday.)

**6.** [1] For `calculate_deadline("2026-04-30", 1, BE, business)` the result is
`2026-05-04` with `shifted = false`. Why is `shifted` false even though the date
is not `start + 1`?

**7.** [1] Why does the code compute Easter with the anonymous Gregorian
computus instead of storing the dates for each year?

**8.** [1] Name the two conditions that make a date a **business day**.

---

## Section 3 — Compliance (5 marks)

**9.** [2] Compute the mod-11 check digit for the first 8 digits `12345678`
(weights `2,7,6,5,4,3,2,1`; digit-sum each product; check =
`(10 - sum % 10) % 10`). Show your total.

**10.** [1] `DE111111110` has a valid shape and ends in a nonzero digit `0`?
Explain why it is rejected.

**11.** [1] `" de123.456.78 8 "` is accepted as valid. Which step makes this
possible, and what is the normalized result?

**12.** [1] Why can `validate_register_number` not verify a check digit?

---

## Section 4 — System design (5 marks)

**13.** [2] Match each situation to its HTTP status:
(a) request body is not valid JSON;
(b) `federal_state` is `"XX"`;
(c) more than 120 requests in a minute from one client;
(d) Postgres is down when the app starts (readiness).
Choose from: 400, 401, 422, 429, 500, 503.

**14.** [1] The redaction audit table stores `payload_sha256` and per-category
counts but never the original text. State the privacy principle this satisfies
and one thing the digest still allows you to do.

**15.** [2] Define "fail open" as used by the rate limiter and the deadline
cache, and give one reason it is the right choice *for this application*.

---

*End of quiz. Total: 20 marks.*
