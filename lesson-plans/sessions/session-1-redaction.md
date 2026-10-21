# Session 1 — The Redaction Automaton (GDPR Data Masking)

**Duration:** 90 minutes  **Primary code:** `backend/src/services/redaction.rs`
**Capstone link:** Task 1 of `challenge.md`  **Lab:** `worksheets/lab-1-redaction.md`

## Learning goal (this session)

Learners can implement a single-pass text redactor for emails, German IBANs, and
phone numbers, and can explain why a *combined* pattern with an *ordering rule*
beats three independent replacements — including one real false-positive bug.

## Success criteria

By the end, learners can:
1. Write a regex that replaces a match with a fixed placeholder.
2. Justify the left-to-right ordering `email | IBAN | phone`.
3. Given `"AZ-2026/0042 2026-05-01"`, predict that **nothing** is redacted and
   explain which two pattern rules prevent it.

## Prior knowledge to activate (2 min)

Ask: "Name every way you can write a phone number." Collect on the board.
Goal: show the variety (`+49 30 …`, `0049 30 …`, `030/…`, `0170 …`) that the
pattern must cover — and the ambiguity that creates.

---

## Sequence

### 0:00–0:08 — Hook: the leak

Show an administrative free-text field (real-ish):

```
Sehr geehrte Frau Müller,
Ihre Bewerbung unter m.mueller@web.de wurde geprueft.
Die Ueberweisung (DE02120300000000202051) ist eingegangen.
Bitte rufen Sie unter 030/5551234 an. Freundliche Gruesse, Referat B4.
```

Ask: "This goes to a third party. What must not leave the building?" Circle
the email, IBAN, phone. State the legal frame: **DSGVO/GDPR Art. 5** — data
minimization. This is a *masking* problem, not a *deletion* problem.

### 0:08–0:20 — Concept: one pass, three classes

Teach from the real code:

- The three placeholder constants — `EMAIL_PLACEHOLDER`, `IBAN_PLACEHOLDER`,
  `PHONE_PLACEHOLDER` (`redaction.rs:29-31`).
- The **single combined regex** in a `LazyLock` (`redaction.rs:11-27`). Emphasize
  *why one pass*: if you replaced emails first and then searched for phones,
  your own placeholder text could be re-matched (`redaction.rs:2-4` documents
  this).
- The **ordering rule**: `email | IBAN | phone`; first alternative wins at a
  given position (`redaction.rs:12`).
- Named capture groups (`?P<email>`, `?P<iban>`, `?P<phone>`) let the code count
  which category matched (`redaction.rs:58-75`) — this powers the per-category
  counts and the audit.

**Board the anatomy of the pattern:**

```
(?P<email> [A-Za-z0-9._%+-]+ @ [A-Za-z0-9.-]+ \. [A-Za-z]{2,} )
| (?P<iban> DE\d{2}\s?(?:\d{4}\s?){4}\d{2} )
| (?P<phone> …international… | \b0[1-9]\d{0,3}[\s\-/]\d{5,} )
```

### 0:20–0:32 — Code walk: the masking loop

Walk `mask_sensitive_data` (`redaction.rs:53-79`):

- It keeps a `last` cursor and copies untouched text between matches
  (`redaction.rs:60`, `:76`) — this is a manual "replace all" that also counts.
- `counts.email += 1` etc. (`redaction.rs:61-69`).
- Result type `RedactionResult { text, counts }` (`redaction.rs:46-50`).

Run the unit tests live: `cargo test --lib redaction`. Point at
`masks_mixed_document_and_counts_per_category` (`redaction.rs:129`) and
`adjacent_entities_do_not_bleed_into_each_other` (`redaction.rs:148`).

### 0:32–0:42 — Concept: the ambiguity trap (the real bug)

Present the incident. The first version of the national phone rule was
`\b0\d{2,4}[\s\-/]\d{3,}`. Ask learners to find the false positive in:

```
AZ-2026/0042 2026-05-01 1.234,56 EUR
```

Answer: `0042 2026` matched — the tool redacted a **file number** as a phone
number, corrupting the reference.

Walk the fix (`redaction.rs:14-24`):
- National numbers never start with `00` (that is the *international* prefix) →
  `0[1-9]` instead of `0\d`.
- The subscriber part needs **≥ 5 digits** (`\d{5,}`), so a 4-digit year like
  `2026` cannot be the tail.

Then name the tradeoff explicitly: **false positive** (mask real text) vs.
**false negative** (leak PII). Under GDPR a false negative is worse — so why is
"≥5 digits" acceptable? Because real German numbers have a subscriber of 5+
digits; `does_not_mask_file_numbers_followed_by_dates` (`redaction.rs:184`) and
`masks_short_but_valid_national_numbers` (`redaction.rs:205`) lock in both sides.

### 0:42–0:48 — Break

### 0:48–1:22 — Lab 1 (pairs)

Hand out `worksheets/lab-1-redaction.md`. Core tasks:
1. Predict-then-run four inputs.
2. Add a new test for a number that must be masked and one that must not.
3. Diagnose a provided *deliberately over-broad* pattern variant.

Circulate; use the Hint ladder before giving answers.

### 1:22–1:30 — Debrief + exit ticket

Exit ticket (3 sentences): "A bigger regex is not always safer because… The two
rules that stop `0042 2026` matching are… In this domain a false negative is
worse than a false positive because…"

## If you are behind (cut here)

Cut 0:32–0:42 to a 3-minute summary and push the false-positive lab task to
homework. Do **not** cut the single-pass concept — Session 4 depends on it.

## Materials

- `backend/src/services/redaction.rs`
- `api.http` (the `1.x` redaction block) for optional live demo
- Worksheet: `worksheets/lab-1-redaction.md`

## Teacher notes

- Keep German terms on the board; they are part of the product.
- The strongest discussion comes from *the tradeoff*, not from regex trivia.
  If learners fixate on regex syntax, redirect to "what leak does this prevent?"
- Non-Rust learners: tell them to read `?P<name>` as "label this group"; the
  loop is ordinary string building.
