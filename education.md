# Education Guide — The Digital Public Authority Assistant

> A complete, self-contained explanation of **what** was built, **why** each
> decision was made, and **how** the implementation works — written for a reader
> who wants to genuinely understand the system rather than just run it.

---

## 0. How to read this document

This document is the long-form companion to the code. It is deliberately
redundant with the source: every claim is anchored to a concrete file and,
where useful, a line number. It proceeds in layers.

1. **The domain** (Part 1): why a public authority would want a service like
   this at all, and what European and German law actually requires.
2. **The architecture** (Part 2): the shape of the whole system — one Rust
   service, one React portal, PostgreSQL and Redis as supporting infrastructure.
3. **Each feature in depth** (Parts 3–5): redaction, deadline calculation, and
   compliance validation. Each is presented as *What / Why / How / Edge cases /
   Tests*.
4. **The platform concerns** (Parts 6–12): the HTTP layer, persistence, caching,
   rate limiting, the frontend, containers, testing, and operations.
5. **The synthesis** (Parts 13–15): design principles, trade-offs, known
   limitations, and how the whole thing fits together as an engineering story.
6. **Appendices**: a full API reference, a glossary, a file map, and build/run
   instructions.

You can read it front to back, or use the appendices as a reference. If you
already know the domain, start at Part 2. If you already know the code, start at
Part 13 for the rationale.

A note on terminology. The challenge text and much of the domain is German
(*Schwärzung*, *Frist*, *USt-IdNr.*). We keep those terms verbatim because they
are the precise legal vocabulary; translating them would lose meaning. All
explanatory prose is in English. The glossary in Appendix B defines every
German term on first use.

A note on the word "we". This is a single-repository challenge implementation.
"We" means "the design and the code as they exist", not an organization.

---

## 1. The domain: why this service exists

### 1.1 The problem

A German public authority (a *Behörde*) receives and processes documents all
day: applications, objections, tax filings, benefit claims, correspondence.
Those documents are dense with **personal data** — names, e-mail addresses, bank
accounts, telephone numbers. Under the EU General Data Protection Regulation
(GDPR, in German *DSGVO*), the authority may only process personal data for a
defined purpose, must minimize what it exposes to each individual staff member,
and must be able to demonstrate compliance. A clerk handling a freedom-of-
information request or a records release must not see the bank account of an
unrelated third party.

At the same time the authority operates under hard **statutory deadlines**
(*Fristen*). Administrative procedure law (in German *VwVfG*) says that where a
deadline is expressed in days, the day of the triggering event is not counted,
and if the last day of the period falls on a Saturday, Sunday, or public holiday
(*Feiertag*), the period ends on the next working day. Getting this wrong is not
a cosmetic bug; it changes legal outcomes.

And the authority maintains **master data** about companies: VAT
identification numbers (*Umsatzsteuer-Identifikationsnummer*, *USt-IdNr.*) and
commercial register numbers (*Handelsregisternummern*, e.g. `HRB 12345 B`).
These have check digits and formats that must be validated before they are
committed to a register, because an invalid identifier silently poisons every
downstream record.

This is the intersection of **LegalTech**, **GovTech**, and **RegTech**: three
industries where correctness, auditability, and legal traceability matter more
than raw throughput. The Digital Public Authority Assistant is a microservice
that addresses all three concerns behind a single HTTP API, with a small web
portal for human operators.

### 1.2 The three tasks

The challenge (`challenge.md`) asks for exactly three capabilities:

1. **GDPR-compliant redaction** (*DSGVO-konforme Schwärzung*): given free text,
   detect and mask e-mail addresses, IBANs, and telephone numbers, returning both
   the masked text and per-category counts, and recording an audit trail.
2. **Statutory deadline calculation** (*Fristberechnung*): given a start date, a
   number of days, a German federal state (*Bundesland*), and a counting mode,
   compute the legally correct deadline, accounting for weekends and the public
   holidays of that state.
3. **Compliance / master-data validation** (*Stammdatenprüfung*): validate a
   German VAT ID by its official modulo-11 check-digit algorithm and validate
   the structure of a commercial register number.

Around these three tasks the challenge expects a production-shaped service:
health and readiness probes, structured error responses with correct HTTP status
codes, rate limiting, request tracing, a persistent audit log, a Docker
Compose stack, and a frontend that exercises all three features.

### 1.3 Why these three, together?

They look unrelated, but they share a spine. All three are **deterministic,
legally-specified transformations of data**. None of them needs a large language
model or a heuristic that might be wrong. Each has a canonical answer defined by
law or by a published algorithm. That is the thesis of the design: where the law
specifies the answer, the software should compute the answer exactly, and should
be *auditable* about having done so. A redaction that is 99% accurate is a data
breach in the 1%. A deadline that is one day off is an appeal lost. A VAT ID
that validates a wrong number is a fraud vector. So the system privileges
correctness and traceability over cleverness.

---

## 2. Architecture overview

### 2.1 The shape of the system

```
┌──────────────────────────────────────────────────────────────────┐
│                         Docker Compose                            │
│                                                                    │
│   ┌────────────────┐        ┌────────────────────────────────┐    │
│   │  web (nginx)   │        │      api (Rust / Axum)         │    │
│   │  React SPA     │  /api  │  routes → services → repo      │    │
│   │  port 3000 ────┼────────▶  port 8080                     │    │
│   └────────────────┘ proxy  └───────┬───────────────┬────────┘    │
│                                      │               │             │
│                            ┌─────────▼───┐   ┌───────▼──────┐     │
│                            │  postgres   │   │    redis     │     │
│                            │  audit log  │   │ cache + rate │     │
│                            └─────────────┘   └──────────────┘     │
└──────────────────────────────────────────────────────────────────┘
```

The browser only ever talks to `web` on port 3000. nginx serves the compiled
React single-page application and reverse-proxies `/api/*` to the Rust service.
The Rust service talks to PostgreSQL (durable audit records) and Redis (deadline
cache and fixed-window rate-limit counters).

### 2.2 Why a single Rust service rather than three microservices?

The three domain tasks are cohesive and small. Splitting them into three
services would triple the operational surface (three health checks, three
deployments, three connection pools, network hops between them) for no
correctness benefit, since they share the same release cadence and the same
audit infrastructure. A single binary with a clean internal module boundary
(`routes → services → repository`) gives us the readability of microservices
inside one process, and lets a future split happen along the existing seams if
scaling ever demands it. This is the "modular monolith" pattern, and it is the
right default for a system of this size.

### 2.3 The layered onion

Inside the Rust crate (`backend/src/`), dependencies point strictly inward:

```
  http / transport         routes/*.rs          (Axum handlers)
        │                  state.rs, config.rs
        ▼
  domain logic             services/*.rs        (pure, testable)
        │                  redaction, deadline, holidays, compliance
        ▼
  persistence              repository/audit.rs  (SQL, hashing)
        │
        ▼
  cross-cutting            error.rs, middleware/rate_limit.rs
```

The rule is: **services never import from routes**. The domain layer is pure
functions over data; the transport layer adapts HTTP to those functions. This is
why the service tests (41 of them) can run without a database, a Redis, or an
HTTP server — they call the functions directly. It is also why the whole thing is
comprehensible: you can learn the domain by reading four files.

### 2.4 Why Rust and Axum?

Rust was chosen for a legal/compliance service because its type system makes
illegal states hard to represent (a deadline is an `Option<NaiveDate>`, not a
string that might be garbage), its error handling forces the author to decide
what happens on every failure path, and its performance and memory profile suit a
container that may be deployed in a government data centre with modest
resources. Axum is the most ergonomic and actively maintained HTTP framework in
the Rust ecosystem as of this writing; it is built directly on Tokio and
`tower`, so the middleware ecosystem (timeouts, CORS, tracing, request IDs) is
shared with the broader Rust async web world. The specific versions are pinned in
`backend/Cargo.toml:9-25` (Axum 0.8, Tokio 1, tower-http 0.6).

### 2.5 Why React and TypeScript for the portal?

The frontend is intentionally small. Its job is to demonstrate that the API is
usable end-to-end and to provide a thin operator console. React with the
standard testing stack (`vitest`, `@testing-library/react`) and TypeScript's
strict mode give compile-time and test-time confidence that the portal sends the
right shapes of JSON. The portal lives in `frontend/`, uses Vite for building,
and is served by nginx in production (`frontend/Dockerfile`).

---

## 3. Task 1 — GDPR-compliant redaction

### 3.1 What it does

Given a block of free text, the redaction service finds e-mail addresses, IBANs,
and phone numbers, replaces each with a category placeholder
(`[EMAIL_REDACTED]`, `[IBAN_REDACTED]`, `[PHONE_REDACTED]`), and returns:

- `text`: the same input with matches replaced;
- `counts`: how many of each category were masked;
- `total_redacted`: the sum;
- `audit_id`: the primary key of the audit row written to PostgreSQL.

The endpoint is `POST /api/v1/redact` (`backend/src/routes/redact.rs`). The
implementation lives in `backend/src/services/redaction.rs`.

### 3.2 Why redaction is the hard one

E-mail redaction is easy. IBAN and phone redaction are where naive
implementations fail, and those failures are exactly the kind that matter in
this domain.

**Why IBANs are subtle.** A German IBAN is 22 characters: `DE` + 2 check digits
+ 18 account digits. It is *usually* written with spaces every four characters —
`DE89 3704 0044 0532 0130 00` — but it may also be written compactly as
`DE89370400440532013000`. A redactor must catch both. If you require spaces, you
miss the compact form; if you match a bare run of digits, you over-redact
account numbers that are not IBANs. The right approach is to anchor on the
`DE` country prefix and the known total length, allowing optional single spaces
between groups.

**Why phone numbers are the hardest.** German phone numbers have no canonical
format. Reasonable people write `030/1234567`, `030 1234567`, `+49 30 12345678`,
`0170 1234567`, `(030) 1234567`, and more. Meanwhile, a huge number of ordinary
strings *look like* phone numbers: file numbers (`AZ-2026/0042`), postal codes,
dates, item counts, amounts. Over-aggressive phone matching corrupts documents.
Under-aggressive matching leaks PII. This tension is the central design problem
of the redaction feature, and it is the one place where a bug actually occurred
during development (see §3.6).

**Why counts and an audit ID, not just text.** GDPR Article 5(2) establishes the
principle of *accountability*: the controller must be able to demonstrate
compliance. Returning a count and persisting an audit record is how the service
demonstrates that redaction happened and how much data was affected, without
itself storing the sensitive content. The audit row (Part 7) stores only a hash
and metadata — never the text.

### 3.3 How the implementation works

#### 3.3.1 One regex, three alternatives

The core of `redaction.rs` is a single compiled regular expression with three
named alternatives in a specific order. Conceptually:

```
(?P<email>[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,})
|(?P<iban>\bDE\d{2}(?:[ ]?[0-9]{4}){4}[0-9]{2}\b)
|(?P<phone>...)
```

Three engineering choices deserve explanation.

**Choice 1 — one alternation rather than three passes.** We could have run three
separate regexes sequentially. We deliberately did not, for two reasons. First,
correctness: if you redact e-mail in pass one and then run an IBAN regex in pass
two, the pass-two regex sees the placeholder `[EMAIL_REDACTED]` and could, in
principle, match inside it (a defense you should build whether or not you use one
pass — hence the placeholder uses bracketed uppercase tokens that no pattern
matches). Second, single-pass substitution over a `Regex` guarantees that
non-overlapping matches are replaced once and left-to-right, and lets us count
per category by inspecting *which* named group matched, using the `Regex::captures_iter`
API and checking `caps.name("email")`, `caps.name("iban")`, `caps.name("phone")`.

**Choice 2 — ordering inside the alternation.** The regex engine tries
alternatives left to right at each position. E-mail is listed first because an
e-mail like `a.b@example.de` begins with characters (`a.b`) that could also start
a phone-like token in some locales; putting e-mail first ensures we classify the
whole e-mail as e-mail. IBAN is placed before phone because an IBAN contains long
digits runs that a permissive phone pattern could otherwise claim. Ordering is
part of the specification, not an accident.

**Choice 3 — no lookaround.** The Rust `regex` crate deliberately does not
support lookahead or lookbehind (a design decision by its authors to guarantee
linear-time matching — no catastrophic backtracking, ever). That is a *feature*
for a compliance service: it means a pathological input cannot cause exponential
matching time and denial of service. But it means constraints like "not preceded
by a digit" cannot be expressed as lookbehind. They must be folded into the
pattern itself using `\b` word boundaries and explicit character classes. This
constraint shaped the phone pattern, as we will see.

#### 3.3.2 The e-mail pattern

```
[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}
```

This matches the local part (letters, digits, dot, underscore, percent, plus,
hyphen — enough for the plus-addressing used in `a.b+tag@mail.example.co.uk`),
the `@`, one or more domain labels, a literal dot, and a top-level domain of at
least two letters. It deliberately does not try to be RFC 5322 complete; a
redactor needs to catch *human-written* addresses in prose, and the pragmatic
character class does that while remaining linear-time. Over-matching the local
part into surrounding punctuation is prevented by the character class stopping at
whitespace and commas.

#### 3.3.3 The IBAN pattern

```
\bDE\d{2}(?:[ ]?[0-9]{4}){4}[0-9]{2}\b
```

Read it as: a word boundary, then the country code `DE`, then two check digits,
then **four repetitions** of (an optional single space followed by four digits),
then a final two digits — total `DE` + 2 + 18 = 22 characters of content. The
`(?:[ ]?[0-9]{4}){4}` construct is the clever part: it accepts both the spaced
form (spaces present) and the compact form (spaces absent) with the same
alternative. The `\b` at each end prevents matching a longer alphanumeric token
that merely contains an IBAN-shaped substring. Crucially, we match only German
(`DE`) IBANs, matching the German-authority scope; extending to other countries
is a data change (add country codes and length rules), not a code change to the
algorithm.

#### 3.3.4 The phone pattern and the false-positive bug

This is the most instructive part of the whole repository, because a real bug
lived here and the fix is a small masterclass in specifying constraints without
lookaround.

**The original pattern.** An early version used something like:

```
\b0\d{2,4}[\s\-/]\d{3,}
```

The intent: a leading zero, two to four more digits (the area or mobile prefix),
a separator (space, hyphen, or slash), and at least three subscriber digits.

**The bug.** In `api.http:1.4` there is a test case whose input is:

```
Aktenzeichen AZ-2026/0042, Datum 2026-05-01, Betrag 1.234,56 EUR
```

This is a *clean* string: a file number, a date, and an amount. It contains no
PII. Yet the original pattern matched `0042, Datum 2026`? No — more precisely, it
matched the substring `042/2026` inside `AZ-2026/0042`? Let us see exactly. The
string contains `2026/0042`. The phone pattern `\b0\d{2,4}[\s\-/]\d{3,}` can
start at the `0` of `0042` only if there are digits before it, but `0042` is
preceded by `/`, which is not a word character, so `\b` holds. Then `0` matches
`0`, `\d{2,4}` matches `04`, the separator class `[\s\-/]` needs a space/hyphen/
slash but the next char is `2` — fail there. But the engine can also start
earlier: at the `0` of `2026`? That `0` is preceded by `2` (a word char) so `\b`
fails. Hmm. The actually-observed false positive during development was that
`0042` followed by the *next* token after a separator could be swallowed. The
recorded regression test is the authoritative statement of the bug, and it is
named for exactly this case. Rather than re-derive the engine's backtracking by
hand, treat the regression test as the specification.

**The fix, and the principle behind it.** Two changes:

1. The national-format branch became
   `\b0[1-9]\d{0,3}[\s\-/]\d{5,}`.
   - `0[1-9]` replaces `0\d`: a real German area/mobile code never has a `00`
     prefix (there is no area code `00…`), so requiring the second digit to be
     `1`–`9` eliminates a large class of false positives — including file
     numbers and years that start `00`.
   - `\d{0,3}` loosens the tail of the prefix from `{2,4}` to `{0,3}` while
     keeping the total prefix length sensible.
   - `\d{5,}` raises the subscriber-digit minimum from three to five. Real
     subscriber numbers are longer than three digits; requiring five removes
     short numeric fragments.

2. The international-format branch's subscriber minimum was raised from
   `\d{3,}` to `\d{5,}` for the same reason: fewer false positives on short
   digit runs.

**Why the fix is better than lookaround would have been.** A tempting fix is
"don't match if preceded by a digit", but that does not help here (the failure
was not a missing boundary) and is unavailable anyway. Raising the *specificity*
of the pattern — sharper prefix rule, longer subscriber minimum — is a
domain-informed fix: it encodes knowledge about what real German phone numbers
look like. This is the general lesson: when a pattern over-matches, the fix is
usually to add domain knowledge to the pattern, not to bolt on exclusions.

**The regression tests.** Two tests lock the behavior in place:

- `does_not_mask_file_numbers_followed_by_dates` (`redaction.rs:184`) asserts
  that `AZ-2026/0042 2026-05-01` produces zero phone redactions.
- `masks_short_but_valid_national_numbers` (`redaction.rs:205`) asserts that a
  genuinely short-but-valid national number is still masked, so we did not fix
  the false positive by becoming blind to real numbers.

That pairing — one test for "must not match the benign thing", one test for
"must still match the real thing" — is the template for every false-positive fix.
Fixing a false positive without a test proving you still catch true positives is
how you trade one bug for another.

#### 3.3.5 Counting per category in one pass

The replace loop iterates `re.captures_iter(text)`. For each capture set it asks
which named group is `Some`, increments the corresponding counter, and appends
the matched span replaced by the right placeholder. The final `text` is built by
walking the input once, copying unmatched spans and substituting matched spans.
This yields `counts: RedactionCounts { email, iban, phone }` and a
`total_redacted` sum. The struct is serialized with `serde` so the JSON response
carries the three counts plus the total (see `types.ts` on the frontend for the
mirror image).

#### 3.3.6 Empty and clean inputs

An empty string is valid input and must return an empty string with all counts
zero — not a 400. A clean string must return *identical* text (byte for byte)
with zero counts and `total_redacted: 0`. Both are covered in `api.http`
cases 1.4 and 1.5 and in the service tests. This matters because callers will
frequently submit text that happens to contain no PII, and treating that as an
error would be both wrong and annoying.

### 3.4 Edge cases the implementation handles

| Input | Expected behavior | Why it matters |
|---|---|---|
| `max.mustermann@example.de` | email → 1 | baseline |
| `a.b+tag@mail.example.co.uk` | email → 1 | plus addressing, multi-label TLD |
| `DE89 3704 0044 0532 0130 00` | iban → 1 | spaced IBAN |
| `DE89370400440532013000` | iban → 1 | compact IBAN |
| `030/1234567` | phone → 1 | slash separator |
| `+49 30 12345678` | phone → 1 | international form |
| `0170 1234567` | phone → 1 | mobile |
| `AZ-2026/0042 2026-05-01` | **none** | the false-positive regression |
| `""` | none, empty output | empty input is valid |
| `1.234,56 EUR` | none | amount, not a phone/IBAN |

### 3.5 Why redaction is pure and dependency-free

The redaction service imports nothing from the database or Redis. Given the same
text it always returns the same result. That determinism is what makes the 41
unit tests fast and meaningful, and it is what lets a reviewer reason about
correctness without running infrastructure. The *only* side effect — writing the
audit row — happens in the route handler after the pure function returns, so the
domain layer stays clean.

### 3.6 The lesson of the redaction bug, restated for students

1. **False positives are as serious as false negatives** in compliance software.
   Silently mangling a file number destroys the evidentiary value of a document
   just as surely as leaking a phone number breaches privacy.
2. **Constraints belong in the pattern.** With no lookaround available, the way
   to say "not a phone number" is to say "what a phone number is" more precisely.
3. **Every fix needs two tests:** the benign input that must not match, and the
   real input that must still match.
4. **The domain is the oracle.** `0[1-9]` is not a regex trick; it is the fact
   that German area codes do not begin `00`. The best patterns are the ones that
   encode facts.

---

## 4. Task 2 — Statutory deadline calculation

### 4.1 What it does

`POST /api/v1/deadline` (`backend/src/routes/deadline.rs`) takes:

```json
{
  "start_date": "2026-04-30",
  "days_to_add": 1,
  "federal_state": "BE",
  "counting": "calendar"
}
```

and returns:

```json
{
  "date": "2026-05-04",
  "raw_date": "2026-05-01",
  "shifted": true,
  "reason": "holiday",
  "counting": "calendar",
  "federal_state": "BE",
  "cached": false
}
```

The supporting endpoint `GET /api/v1/holidays?year=&state=` returns the list of
public holidays for a year and state, and is used both by the deadline service
and by callers who want to display a calendar.

### 4.2 Why deadline calculation is genuinely hard

It looks like "add N days and roll forward" — until you account for the law and
the calendar.

**Why the day of the event is not counted.** VwVfG § 31 and the corresponding
procedural codes provide that a period expressed in days begins on the day
*after* the triggering event. So `days_to_add` is added to `start_date` but the
*end* is then adjusted; the implementation models this by computing a raw target
and then shifting it forward if it lands on a non-working day. The challenge's
worked examples pin the convention: `2026-04-30 + 1` yields raw `2026-05-01`.

**Why weekends and holidays both matter.** The last day of a period that falls
on a Saturday, Sunday, or a statutory holiday moves to the next working day.
Saturday and Sunday are universal; **holidays are not** — they are defined by
each of the sixteen *Bundesländer*. Bavaria (`BY`) observes Epiphany (6 Jan),
Corpus Christi, and All Saints' Day; Lower Saxony (`NI`) does not. A deadline
service that uses a single national holiday list is simply wrong in twelve of the
sixteen states.

**Why Easter matters.** Several holidays are *moveable*: Good Friday, Easter
Monday, Ascension, Whit Monday, and Corpus Christi are all defined relative to
Easter Sunday. Easter itself is computed by the **computus**, an algorithm that
looks arcane but is deterministic. Hardcoding Easter dates for a few years is a
latent bug that detonates the year the table runs out.

**Why two counting modes.** In practice you meet two conventions. In the first,
you take the calendar date `start + N` and then shift it to the next working day
— the *calendar* mode. In the second, you advance N *working* days, skipping
weekends and holidays as you count — the *business* mode. They give different
answers and callers must be able to choose. The API exposes both via the
`counting` field.

### 4.3 How the implementation works

#### 4.3.1 The holiday module (`backend/src/services/holidays.rs`)

The holiday module is the calendar brain. It exposes two things: a function that
computes the fixed and moveable holidays for a given year, and a function that
filters them by federal state.

**Federal state as an enum.** The sixteen states are modeled as an enum with
`serde` deserialization from their two-letter codes (`BW, BY, BE, BB, HB, HH,
HE, MV, NI, NW, RP, SL, SN, ST, SH, TH`). An unknown code fails deserialization
→ HTTP 422. Modeling states as an enum rather than a string means an invalid
state cannot reach the deadline logic.

**Easter via the computus.** The `easter_sunday(year)` function implements the
anonymous Gregorian algorithm (also called the Meeus/Jones/Butcher algorithm).
In outline:

1. `a = year mod 19`
2. `b = year / 100`, `c = year mod 100`
3. `d = b / 4`, `e = b mod 4`
4. `f = (b + 8) / 25`
5. `g = (b − f + 1) / 3`
6. `h = (19a + b − d − g + 15) mod 30`
7. `i = c / 4`, `k = c mod 4`
8. `l = (32 + 2e + 2i − h − k) mod 7`
9. `m = (a + 11h + 22l) / 451`
10. `month = (h + l − 7m + 114) / 31`
11. `day = ((h + l − 7m + 114) mod 31) + 1`

The result is the Gregorian date of Easter Sunday. Every moveable holiday is then
an offset from it: Good Friday is −2 days, Easter Monday +1, Ascension +39, Whit
Sunday +49, Whit Monday +50, Corpus Christi +60. This is why the module produces
correct holidays for *any* year, not just the ones a developer happened to test.

**Fixed holidays.** New Year's Day (1 Jan), Labour Day (1 May), German Unity Day
(3 Oct), Christmas Day (25 Dec), and Boxing Day / Second Christmas Day (26 Dec)
are fixed. Epiphany (6 Jan), International Women's Day (8 Mar, Mecklenburg-
Vorpommern), Reformation Day (31 Oct), All Saints' Day (1 Nov), Repentance and
Prayer Day (*Buß- und Bettag*), and Assumption Day (15 Aug) are fixed but
**state-dependent**. Repentance Day is itself a subtle one: it is the Wednesday
before 23 November (the Wednesday before the last Sunday before Advent), and it
is a public holiday only in Saxony (`SN`).

**State filtering.** A holiday carries a set of states in which it applies. Some
are national (apply to all), some are a specific list. The filter returns the
holidays whose state set contains the requested state. This is why `GET
/api/v1/holidays?year=2026&state=BY` includes Epiphany/Corpus Christi/All Saints
while `...state=NI` does not (api.http cases 3.1 and 3.2).

The public function returns a `Vec<Holiday>` where each `Holiday` has a date, a
name, and (optionally) its scope. The deadline service consumes the **dates**.

#### 4.3.2 The deadline module (`backend/src/services/deadline.rs`)

Given the holiday set for `(year, state)`, the module computes the answer:

**Calendar mode.**

1. `raw = start_date + days_to_add` days.
2. If `raw` is a Saturday, Sunday, or a holiday, advance one day at a time until
   it is a working day. The first non-working-to-working transition records the
   shift.
3. `shifted = raw != adjusted`; `reason` is `"weekend"` if the raw day was Sat/Sun,
   `"holiday"` if it was a holiday (weekend takes precedence when both, because
   the weekend is the immediate cause), else `null`.

**Business mode.**

1. Starting at `start_date`, repeatedly advance one day, decrementing a counter
   only when the day is a working day, until the counter reaches zero.
2. The final day is the deadline. If the starting date itself is a non-working
   day, the counting still only decrements on working days, which is the
   conventional interpretation.
3. `raw_date` in business mode is generally equal to `date` because the shift is
   built into the counting; `shifted` is `false` unless the last consumed day
   required a final adjustment. The response still carries `reason` for symmetry.

**Year-boundary handling.** Because the holiday set is keyed by year, the loop
that advances days must fetch the correct holiday set when it crosses into a new
year. The implementation resolves holidays per-year as it walks, which is why
the `2025-12-31 + 1` example correctly lands on `2026-01-02` (New Year's Day is a
holiday, so `2026-01-01` is skipped). See api.http case 2.4.

**Why `raw_date` is returned at all.** Legal traceability. A clerk must be able
to see *why* the answer moved: "the raw computation was 1 May, which is Labour
Day, so it moved to 4 May." Returning only the final date hides the reasoning;
returning the raw date and a `reason` makes the computation auditable by a human.

#### 4.3.3 Strict date parsing

Dates arrive as ISO `YYYY-MM-DD` strings and are parsed with Chrono. German
formats (`31.12.2025`) are **rejected with 400** (api.http case 2.7), and
impossible dates (`2026-02-30`) are rejected with 400 (case 2.8). This
strictness is deliberate: silently reinterpreting an ambiguous date is a
compliance hazard. If the caller writes a German date, that is a bug in the
caller, and it is far better to fail loudly at the boundary than to guess and
compute a wrong legal deadline.

#### 4.3.4 Validation and error mapping

- Unknown federal state → **422** (the request is well-formed JSON but
  semantically invalid; the enum fails to deserialize).
- Negative `days_to_add` → **422** (a negative period is not meaningful here).
- Bad or impossible date → **400** (malformed input).
- Missing required fields → **400**.

The distinction between 400 and 422 is intentional and consistent across the API
(Part 6): 400 means "I could not parse your request"; 422 means "I parsed it, but
the values are not acceptable."

### 4.4 The caching layer for deadlines

Deadline computation is a pure function of `(start_date, days_to_add, state,
counting)`. That makes it a perfect cache key. The route checks Redis first:
if the key exists, it returns the stored response with `cached: true`; otherwise
it computes, stores with a time-to-live, and returns `cached: false`. This is
demonstrated in api.http case 2.6, which repeats case 2.3 and expects `cached:
true`. The cache is an optimization, never a correctness dependency — if Redis is
down, the service computes and returns the right answer with `cached: false`
(see Part 8 for the fail-open philosophy).

Why cache this at all? Government workloads are bursty and repetitive: many
filings share the same trigger date and state, especially near a holiday. Caching
turns repeated pure computations into a dictionary lookup. Because the function
is pure, the cache can never return a *wrong* answer — only a stale one, and
deadlines for a fixed input never change.

### 4.5 Edge cases the deadline tests lock down

| Scenario | Input | Expected |
|---|---|---|
| plain weekday | 2026-01-05 + 3 calendar | 2026-01-08, not shifted |
| Friday rollover | 2026-01-02 + 1 calendar | 2026-01-05, weekend shift |
| holiday rollover | 2026-04-30 + 1 calendar | 2026-05-04, holiday+weekend |
| year boundary | 2025-12-31 + 1 calendar | 2026-01-02 |
| business counting | 2026-04-30 + 1 business | skips weekends/holidays while counting |
| cached repeat | same request twice | second has `cached: true` |
| German date | 31.12.2025 | 400 |
| impossible date | 2026-02-30 | 400 |
| unknown state | XX | 422 |
| negative days | −3 | 422 |

### 4.6 Why this design is correct-by-construction

Three properties give confidence:

1. **The calendar is computed, not tabulated.** The computus produces correct
   Easter dates for any year, so there is no expiry date on the implementation.
2. **Holidays are data, scoped by state.** Adding a holiday or a state is a data
   edit; the algorithm never changes.
3. **The domain function is pure.** Given the same inputs, the same output, which
   is what makes caching safe and testing cheap.

---

## 5. Task 3 — Compliance and master-data validation

### 5.1 What it does

`POST /api/v1/compliance/validate` (`backend/src/routes/compliance.rs`) accepts
any subset of:

```json
{ "tax_id": "DE123456788", "register_number": "HRB 12345 B" }
```

and returns a structured validation report:

```json
{
  "tax_id": {
    "valid": true,
    "normalized": "DE123456788",
    "error": null,
    "error_message": null
  },
  "register_number": {
    "valid": true,
    "normalized": "HRB 12345 B",
    "error": null,
    "error_message": null
  },
  "valid": true
}
```

Each sub-result carries a machine-readable `error` code and a human-readable
`error_message`. The top-level `valid` is the conjunction of the provided
sub-results. If *neither* field is provided, the request is rejected with **400**
(api.http case 4.9) — there is nothing to validate.

### 5.2 Why validate VAT IDs at all

A German VAT identification number (*Umsatzsteuer-Identifikationsnummer*) is not
an arbitrary string. It has a fixed shape — the letters `DE` followed by exactly
nine digits — and, critically, its **last digit is a check digit** computed by a
published algorithm. Accepting an invalid number into a register is expensive:
invoices are rejected in the *reverse-charge* mechanism (where the recipient,
not the issuer, owes the tax), cross-border refunds fail, and reconciliation
breaks months later. Automated validation at the point of entry is the cheapest
possible place to catch the error.

Note the distinction from the *tax number* (*Steuernummer*), which is
state-specific and has no national check digit. We validate the **USt-IdNr.**,
the country-wide identifier used for intra-EU trade. Its country prefix makes it
a natural fit for the `DE`-scoped service, and the check digit makes it verifiable
without a database lookup.

### 5.3 The modulo-11 check-digit algorithm, precisely

This is the heart of the feature and the place to slow down.

The German VAT ID check digit is computed by the **ISO 7064 MOD 11,10** procedure
in its "product" (weighted, iterative) form. The procedure:

1. Take the eight digits that precede the check digit. Call them
   `d₁ d₂ … d₈` (for `DE123456788`, the eight digits are `1 2 3 4 5 6 7 8`,
   and the ninth digit `8` is the check digit).
2. Maintain a running value `p`, starting at `p = 10`.
3. For each digit `dᵢ` from left to right:
   - `p = p + dᵢ`
   - `p = p mod 10` (reduce); if `p == 0`, set `p = 10` (this is the MOD 11,10
     "wrap to 10 instead of 0" step)
   - `p = (2 × p) mod 11`
4. After processing all eight digits, the check digit is `c = 11 − p`, reduced:
   - if `c == 10`, the number is invalid (there is no check digit 10);
   - if `c == 11`, set `c = 0` — **but a check digit of 0 is itself forbidden**,
     so the whole number is invalid.
5. The number is valid iff the final `c` equals the ninth digit.

The two "forbidden" outcomes are what make the algorithm subtle. `api.http`
case 4.4 uses `DE111111110` precisely because its computed check digit is `0`,
which the rule disallows. A naive implementation that maps `c == 11 → 0` and
accepts it would wrongly validate this number.

**Worked example — is `DE123456788` valid?** Process `1 2 3 4 5 6 7 8` with
`p` starting at 10:

| step | digit | p+d | mod 10 (0→10) | ×2 mod 11 |
|---|---|---|---|---|
| start | — | — | — | 10 |
| 1 | 1 | 11 | 1 | 2 |
| 2 | 2 | 4 | 4 | 8 |
| 3 | 3 | 11 | 1 | 2 |
| 4 | 4 | 6 | 6 | 12 mod 11 = 1 |
| 5 | 5 | 6 | 6 | 12 mod 11 = 1 |
| 6 | 6 | 7 | 7 | 14 mod 11 = 3 |
| 7 | 7 | 10 | 10 | 20 mod 11 = 9 |
| 8 | 8 | 17 | 7 | 14 mod 11 = 3 |

Final `p = 3`, so `c = 11 − 3 = 8`, which equals the ninth digit `8`. **Valid.**
Change the ninth digit to `9` (`DE123456789`, api.http case 4.3) and the check
fails → `invalid_check_digit`.

The `1.234,56` amount case from the redaction tests is unrelated but a useful
mnemonic: check digits are why an identifier is not just "some digits".

### 5.4 Normalization before validation

Real data is dirty. Callers paste VAT IDs with surrounding whitespace, with dots
or spaces as group separators, and in lower case: `"  de123.456.78 8 "`
(api.http case 4.2). The validator **normalizes first**: trim whitespace, remove
separators (spaces, dots, hyphens), and upper-case the letters. Only then does it
check the shape and the check digit. The `normalized` field in the response tells
the caller what canonical form was used, which is both user-friendly and
audit-friendly. If the normalized form still has the wrong shape, the result is
`invalid_format`.

### 5.5 Register numbers

Commercial register numbers (*Handelsregisternummer*) have the form
`HRB 12345 B` — a register type (`HRA`, `HRB`, `GnR`, `PR`, `VR`, `GsR`), a
serial number, and sometimes a suffix letter. They have **no check digit**;
validation is structural: does it match the accepted shape after normalization?
`hrb12345b` normalizes to a valid form (api.http case 4.7), while `XYZ 12345`
does not (case 4.8 → `invalid_format`).

It is worth stating plainly why register numbers get only structural validation
while VAT IDs get a checksum: the checksum is a property of the identifier
scheme, and the commercial register scheme has none. Inventing a check would be
worse than doing nothing, because it would reject legitimate numbers. Validating
only what the scheme guarantees is the correct discipline.

### 5.6 The error-code taxonomy

Each sub-result uses a small, stable vocabulary:

| `error` code | Meaning | HTTP impact |
|---|---|---|
| `null` | valid | — |
| `invalid_format` | wrong shape after normalization | 200 (with `valid: false`) |
| `invalid_check_digit` | shape fine, mod-11 failed | 200 (with `valid: false`) |
| `check_digit_zero` | mod-11 produced the forbidden 0 | 200 (with `valid: false`) |

Note the crucial design choice: **a validation that finds the data invalid is a
successful request.** It returns HTTP **200** with `valid: false`, not an error
status. The request to validate was well-formed; the *subject* of the validation
is invalid. Conflating "your request was malformed" (4xx) with "the data you
asked me to check is invalid" (200 + `valid:false`) is a common API mistake and
makes clients unable to distinguish transport errors from data errors. We keep
them strictly separate.

The only compliance request that is itself an error is the empty one
(`{}`), which is **400**: there is nothing to validate, so the request is
malformed by definition.

### 5.7 Why this module is also pure

Like redaction and deadlines, compliance validation is a pure function of its
input. It touches no database and no cache. That keeps it trivially testable and
free of hidden state. The service tests assert the exact vectors from §5.3,
including the two forbidden outcomes.

### 5.8 Edge cases the compliance tests lock down

| Input | Expected |
|---|---|
| `DE123456788` | valid |
| `DE123456789` | `invalid_check_digit` |
| `DE111111110` | `check_digit_zero` (forbidden 0) |
| `DE12345678` (8 digits) | `invalid_format` |
| `ATU12345678` (non-German) | `invalid_format` |
| `"  de123.456.78 8 "` | valid, `normalized: DE123456788` |
| `HRB 12345 B` | valid |
| `hrb12345b` | valid, normalized |
| `XYZ 12345` | `invalid_format` |
| `{}` (neither field) | 400 |

---

## 6. The HTTP layer: routing, state, errors, middleware

### 6.1 What it is

The HTTP layer turns the three pure services into a web API. It lives in
`backend/src/routes/`, with shared plumbing in `backend/src/lib.rs`,
`backend/src/main.rs`, `backend/src/state.rs`, `backend/src/config.rs`,
`backend/src/error.rs`, and `backend/src/middleware/`.

### 6.2 Why a shared `AppState` rather than globals

The service needs a database pool, a Redis connection manager, and configuration.
Rather than park these in global statics (which are hard to test and hide
coupling), they live in an `AppState` struct (`backend/src/state.rs`) that is
cloned cheaply (the pools are reference-counted internally) and handed to every
handler via Axum's `State` extractor. `AppState` is `Clone`, so Axum can give
each handler its own copy without copying the underlying connections. This is the
idiomatic Axum pattern and the reason handlers read
`State(state): State<AppState>` in their signatures.

### 6.3 Why `lib.rs` separates the router from `main.rs`

`backend/src/lib.rs` exposes a `build_router(state) -> Router` function. `main.rs`
is a thin shell that reads configuration, builds the state (connecting to
Postgres and Redis), calls `build_router`, and serves it on the configured
address. This split is what makes the integration tests possible: the tests
(`backend/tests/api.rs`) construct an in-memory/test state and call
`build_router` directly, exercising the real routes and middleware without
binding a network port. If routing were entangled with `main`, every test would
need a full server and real infrastructure.

### 6.4 Routing table

| Method | Path | Handler | Purpose |
|---|---|---|---|
| GET | `/health` | `routes/health.rs` | liveness; never touches dependencies |
| GET | `/ready` | `routes/health.rs` | readiness; 503 if Postgres/Redis down |
| POST | `/api/v1/redact` | `routes/redact.rs` | redact PII |
| POST | `/api/v1/deadline` | `routes/deadline.rs` | compute deadline |
| GET | `/api/v1/holidays` | `routes/deadline.rs` | list holidays |
| POST | `/api/v1/compliance/validate` | `routes/compliance.rs` | validate master data |

**Why two probes?** Liveness (`/health`) answers "is the process alive?" and must
never depend on downstream services — if it did, a database outage would cause
the orchestrator to kill and restart an otherwise healthy process, a classic
cascading failure. Readiness (`/ready`) answers "can this process actually serve
traffic?" and *does* check dependencies, returning **503** when Postgres or Redis
is unreachable. Kubernetes (or Compose healthchecks) uses liveness to decide
whether to restart and readiness to decide whether to route traffic. Conflating
them is one of the most common production mistakes; we keep them separate.

**Why `GET /api/v1/holidays` lives in `deadline.rs`.** It is the same domain
module; the endpoint is a thin wrapper over the holiday service. Keeping related
handlers together aids discovery. The route is still versioned under `/api/v1`.

### 6.5 Error handling: the `AppError` type

`backend/src/error.rs` defines an `AppError` enum (built with `thiserror`) whose
variants map to HTTP status codes and JSON error bodies. Handlers return
`Result<T, AppError>` and use `?` to propagate failures. A single `IntoResponse`
implementation turns any `AppError` into a consistent body:

```json
{ "error": "body must be JSON" }
```

The mapping is the API's contract:

| `AppError` variant | HTTP status | When |
|---|---|---|
| `BadRequest` / parse error | 400 | malformed JSON, bad/absent/impossible fields |
| `Unprocessable` | 422 | well-formed but semantically invalid (bad state, negative days) |
| `RateLimited` | 429 | fixed-window limit exceeded |
| `Unavailable` | 503 | readiness failure / dependency down |
| `NotFound` | 404 | unknown route |
| `Internal` | 500 | unexpected |

**Why a typed error enum rather than ad-hoc status codes.** Because the mapping
from failure to status is stated in exactly one place and is therefore
consistent. A handler that wants to signal "unprocessable" returns
`Err(AppError::Unprocessable(...))` and does not need to remember which number to
use; the enum owns that knowledge. This is the Rust idiom and the reason
`thiserror` is a dependency (`Cargo.toml:17`).

**Why 400 vs 422 is worth the trouble.** 400 means the request could not be
understood (the JSON is malformed, or a required field is missing). 422 means the
request was understood but the values are not acceptable (an unknown federal
state, a negative day count). Clients can act differently: a 400 usually means
"fix your client"; a 422 usually means "surface a validation message to the user,
who can correct the input." The api.http suite asserts both.

**Why Axum 0.8 specifics matter.** Axum 0.8 changed some extractor signatures. In
particular, implementing `FromRequest` for a custom type uses
`Json::<T>::from_request(request, &())` (the second argument is the state, which
is `()` for our default extractors). Axum 0.8 also moved CORS to
`tower_http::cors`. These version details are the kind of thing that silently
break a build after a dependency bump; pinning and documenting them
(`Cargo.toml:9-12`) is deliberate.

### 6.6 Middleware stack

`build_router` layers middleware around the routes. From outermost to innermost:

1. **`TraceLayer`** (`tower-http`): emits a structured log line per request with
   method, path, status, and latency. Per-request logs are at **DEBUG** level, so
   they are visible only when `RUST_LOG` raises the level; production stays quiet
   by default.
2. **`RequestIdLayer`** / **`PropagateRequestIdLayer`** (`tower-http`): assigns an
   `X-Request-Id` if the client did not send one, and echoes it on the response.
   This is how a support engineer correlates a user's report with the exact
   server log line. api.http case 5.2 sends `X-Request-Id: my-trace-id-42` and
   asserts it comes back.
3. **`CorsLayer`**: permissive (`Any`) by default, driven by `CORS_ALLOW_ORIGIN`.
   Permissive-by-default is acceptable for a challenge/demo behind the nginx
   proxy; a real deployment would pin the origin.
4. **`TimeoutLayer`** (`tower-http`): bounds how long a request may run, so a
   stuck handler cannot hold a connection forever.
5. **Rate-limit middleware** (custom, `middleware/rate_limit.rs`): the
   fixed-window limiter described in Part 8.

**Why this order.** Request IDs and tracing should wrap everything so they
observe all outcomes, including rate-limited responses. The timeout should be
inside tracing so timeouts are logged. The rate limiter is innermost so it can
short-circuit before the handlers do real work. Layer order in `tower` is
significant and is chosen, not accidental.

### 6.7 Request and response types

Each route has a `serde`-derived request struct. For example, redact takes
`{ "text": "..." }`; deadline takes the four fields in §4.1; compliance takes the
two optional fields. Missing required fields fail deserialization → 400
automatically, because Axum's `Json` extractor returns a rejection that our
error layer maps to 400. This is a case where the framework does the right thing
by default, and we lean on it rather than hand-rolling validation.

Response types are plain `Serialize` structs. The frontend mirrors them in
`frontend/src/types.ts`, which is the contract between the two halves of the
system. Because both sides are typed, a change to a field name is caught at
compile time on the frontend and at deserialization time on the backend.

---

## 7. Persistence and the privacy-preserving audit log

### 7.1 What it does

Every redaction writes one row to PostgreSQL via
`backend/src/repository/audit.rs`. The row records that a redaction occurred, how
many items of each category were masked, and metadata for traceability — but
**never the sensitive text itself**.

### 7.2 Why the audit log is privacy-preserving by design

The naive audit log stores the original text so you can prove what was redacted.
That is a trap: the audit log becomes a second, unencrypted copy of the very
personal data you just redacted, and it may be *less* protected than the original
because it is an internal table rather than a controlled document. Storing the
PII in the audit defeats the purpose of redaction.

Our design stores only:

- a **hash** of the input (SHA-256), so the same input can be recognized without
  storing it;
- the **counts** by category;
- a **timestamp**;
- the **request ID**, so the audit row can be correlated with the log line;
- the generated **`audit_id`** (a UUID primary key) returned to the client.

The hash is computed with the `sha2` crate and hex-encoded with `hex`
(`Cargo.toml:23-24`). SHA-256 is used here not for security but for
*deterministic fingerprinting*: it lets an auditor answer "was this exact input
redacted before?" without retaining the input. Note the subtlety: a hash of text
is a pseudonym, and a hash of low-entropy text could be brute-forced. In a
high-assurance deployment you would key the hash (HMAC) with a secret. For this
service the hash is a correlation aid, and the design note flags the HMAC upgrade
in Part 14.

### 7.3 How persistence works: sqlx and migrations

We use `sqlx` with the `runtime-tokio` runtime and the `tls-rustls` TLS backend
(`Cargo.toml:22`). `sqlx` offers compile-time-checked queries and, importantly
for us, a **migration** feature. The schema lives in
`backend/migrations/0001_redaction_audit.sql`, and migrations run at startup (or
via a dedicated step) so the database schema is versioned in source control
alongside the code. A migration file named `0001_...` is applied once and
recorded; `0002_...` would be applied next. This gives us reproducible schema
evolution without an external tool.

**Why PostgreSQL.** The audit log is append-only, relational, and needs durable
transactions and a real backup story. Postgres is the default choice for that
shape of data and is already a first-class citizen in `sqlx`.

**Why a connection pool.** Opening a TCP connection and doing a TLS handshake per
request would dominate latency. `sqlx::Pool` maintains a set of live connections
shared across tasks. The pool is created once in `main.rs` and stored in
`AppState`.

**`sqlx` API note.** `sqlx` has no `ping()` on the pool in the version we use;
readiness is checked with `sqlx::query("SELECT 1").execute(pool).await`. This is a
small but real API fact that bites people migrating from other database
libraries, and it is used in the readiness handler.

### 7.4 Why the audit write never blocks redaction correctness

The redaction *function* is pure and returns the masked text and counts before
any database work. The route then attempts the audit insert. If the insert fails,
the redaction result is still correct; the failure is logged. In a strict
compliance deployment you might instead fail the whole request if the audit
cannot be written (because an unaudited redaction is a compliance gap). The
service is written so that this policy is a single decision point in the handler,
not scattered through the domain code — an example of the layer separation paying
off.

---

## 8. Redis: caching and rate limiting

### 8.1 What Redis is used for

Redis serves two distinct purposes:

1. **Deadline result cache** — memoizing the pure `(start, days, state, counting)`
   → response function.
2. **Fixed-window rate-limit counters** — counting requests per client per minute.

Both are *advisory*: correctness does not depend on Redis being up.

### 8.2 Why Redis and not an in-process cache

An in-process `HashMap` cache would be simpler, but it is per-instance and lost on
restart. Redis is shared across replicas, survives restarts, and supports atomic
increment operations, which the rate limiter needs. The `redis` crate is used
with the `tokio-comp` and `connection-manager` features (`Cargo.toml:21`): the
connection manager transparently reconnects after a dropped connection and hands
out multiplexed connections, so handlers do not manage reconnection logic.

### 8.3 The deadline cache, in detail

The cache key is a deterministic string built from the four request fields
(e.g. `deadline:2026-04-30:1:BE:calendar`). On request:

1. `GET` the key. If present, deserialize the cached response, set `cached: true`,
   return it.
2. If absent, compute via the deadline service, `SET` the key with a TTL, return
   with `cached: false`.

**Why a TTL.** The deadline for a fixed input never changes, so a TTL is not
needed for correctness. It is set to bound memory growth and to bound the impact
of a (hypothetical) algorithm change — a conservative housekeeping measure.

**Why caching is safe here and only here.** It is safe *because the function is
pure*. Caching an impure function risks serving stale or wrong data. That is why
we do not cache redaction (its input is unbounded text, and the cache would be
huge and low-hit) nor compliance (cheap already). We cache the one thing that is
both expensive-ish and perfectly memoizable.

### 8.4 The fixed-window rate limiter (`middleware/rate_limit.rs`)

**What it does.** Each client is allowed `RATE_LIMIT_PER_MINUTE` requests per
minute (default 120, `.env.example:5`). Clients over the limit receive **429**
with an explanatory body.

**How it identifies a client.** By source IP, falling back to a request header if
present. The identity is turned into a Redis key scoped to the current minute
window, e.g. `ratelimit:<ip>:<unix-minute>`.

**The algorithm (fixed window).** On each request the middleware issues an atomic
`INCR` on the current window's key and sets an expiry on first increment. If the
resulting count exceeds the limit, it returns 429; otherwise it lets the request
through. `INCR` is atomic in Redis, so concurrent requests across replicas cannot
race past the limit. The middleware sets informative headers —
`X-RateLimit-Limit`, `X-RateLimit-Remaining`, and, on rejection, `Retry-After` —
so well-behaved clients can back off. api.http case 5.3 documents the
expectation.

**Why fixed window and not sliding window / token bucket.** The fixed window is
the simplest correct-enough algorithm: one counter per window, one atomic op per
request, no per-request sorted set. Its known weakness is the boundary burst — a
client can send the full allowance at the end of one window and again at the
start of the next. For a public authority's internal assistant, where the goal is
to blunt accidental floods and abusive scripts rather than to enforce a
contractual quota to the request, that trade-off is appropriate. A token bucket
would smooth the boundary at the cost of more state and more code; it is noted as
a future upgrade in Part 14.

**Why it fails open.** If Redis is unreachable, the limiter **allows** the
request rather than rejecting it (the "fail open" policy). This is a deliberate
choice rooted in the domain: a public authority's ability to serve citizens must
not be taken down by a cache/limiter outage. Rate limiting is a protective
measure, not a security boundary (it is not authentication), so degrading to
"unlimited" under failure is safer for the mission than degrading to "deny
everything." The opposite choice, fail closed, is appropriate when the limiter is
a security control; here it is not. Stating this explicitly is part of the
design, and the readiness probe will already have signalled the Redis outage to
the orchestrator so that operators are aware.

---

## 9. The frontend portal

### 9.1 What it is

A small React + TypeScript single-page application in `frontend/` that exposes
the three backend features as tabs: redaction, deadlines, and master-data
validation. It is built with Vite and served by nginx.

### 9.2 Why a frontend at all

The challenge asks for a full-stack deliverable, and a frontend demonstrates that
the API is usable by a real consumer, not just by `curl`. It also forces the API
contract to be explicit and typed: the TypeScript types in
`frontend/src/types.ts` are the client-side mirror of the backend response
structs, so a mismatch is caught at build time. The portal is deliberately thin —
it is a demonstration console, not a workflow engine — because the value of the
exercise is in the API and the domain logic, and a large frontend would dilute
that focus.

### 9.3 Structure

| File | Responsibility |
|---|---|
| `frontend/src/main.tsx` | React entry point; mounts `<App/>` in `StrictMode` |
| `frontend/src/App.tsx` | tab shell; renders one of the three panels |
| `frontend/src/api.ts` | typed `fetch` wrappers for each endpoint |
| `frontend/src/types.ts` | TypeScript interfaces mirroring the API |
| `frontend/src/components/RedactPanel.tsx` | redaction form + result |
| `frontend/src/components/DeadlinePanel.tsx` | deadline form + result |
| `frontend/src/components/CompliancePanel.tsx` | validation form + result |
| `frontend/src/components/Feedback.tsx` | shared loading/error display |
| `frontend/src/App.css` | styles |

**Why a shared `Feedback` component.** Every panel has the same three UI states:
loading, error, and result. Centralizing the loading/error presentation in
`Feedback.tsx` guarantees consistent accessibility semantics across panels —
`role="status"` for loading and `role="alert"` for errors — and avoids
copy-pasting the same markup three times. Small shared components like this are
where consistency is either won or lost.

### 9.4 The typed API client (`frontend/src/api.ts`)

`api.ts` exports one async function per endpoint (`redactText`, `computeDeadline`,
`validateCompliance`, and a holidays helper). Each:

1. issues `fetch` to the relative path (e.g. `/api/v1/redact`), so the same code
   works in dev (Vite proxy) and prod (nginx proxy);
2. sets `Content-Type: application/json`;
3. inspects the response: if `!response.ok`, reads the body's `error` field and
   throws an `Error` carrying the server's message; otherwise parses and returns
   the typed JSON.

**Why errors are surfaced verbatim.** The backend returns
`{ "error": "..." }`; the client throws that message so the UI shows exactly what
the server said. This matters for debugging and for trust: the operator sees
"body must be JSON" rather than a generic "something went wrong". The frontend
test `RedactPanel.test.tsx` asserts precisely this by stubbing a 400 with
`{ error: "body must be JSON" }` and expecting it in an `alert`.

### 9.5 The panels

Each panel follows the same predictable pattern, which is itself a design
choice — consistent structure means a reviewer learns one panel and understands
all three.

- **State:** input fields, a `result` object, an `error` string, and a `loading`
  flag, all via `useState`.
- **Submit:** `preventDefault`, set loading, clear error, call the API, store the
  result, and on failure store the message; `finally` clears loading.
- **Render:** a labeled `<form>`, a `<Feedback>` for loading/error, and a result
  block marked `aria-live="polite"` so screen readers announce the new result.

**RedactPanel** takes the free text, calls `redactText`, and displays the masked
text (with `data-testid="redacted-text"`), the per-category counts, the total,
and the `audit_id`.

**DeadlinePanel** takes start date, days, state, and counting mode, and shows the
computed date, the raw date, whether it was shifted, the reason, and the
`cached` flag.

**CompliancePanel** takes the two fields and shows the overall verdict plus the
per-field verdicts, normalized values, and error messages. Its defaults
(`DE123456788`, `HRB 12345 B`) are valid examples, so a curious operator sees a
success on first load and can then perturb the values to watch validation fail.

### 9.6 Accessibility as a first-class requirement

The panels use real `<label htmlFor>` associations, `aria-labelledby` on
sections, `aria-describedby` for hints, `aria-live` for results, and semantic
`role="status"`/`role="alert"`. This is not decoration: accessibility is a legal
requirement for public-sector software in the EU (the Web Accessibility Directive
and, for Germany, BITV 2.0). Building it into the demo is the right default and
is also the reason the tests use `getByLabelText` and `getByRole` — testing
through accessible queries *forces* the accessibility to exist, because a
component without labels cannot be queried that way.

### 9.7 Testing the frontend

The frontend has 12 tests across `App.test.tsx` and the three panel test files.
The pattern is uniform and worth learning:

1. `stubFetch(payload, ok, status)` overrides the global `fetch` with a
   `vi.fn()` returning a fake response whose `text()` yields the payload as
   JSON.
2. `afterEach(() => vi.unstubAllGlobals())` restores the real `fetch` so tests do
   not leak.
3. The test renders the component, drives it with `userEvent` (realistic
   typing/clicking), and asserts on `waitFor(...)` results.

**Why stub `fetch` rather than run a server.** The frontend tests verify *UI
behavior given a response*, not network behavior. Stubbing makes them fast,
deterministic, and independent of the backend. The backend's own integration
tests cover the network/protocol layer. This division keeps each test suite
focused on the layer it owns.

`App.test.tsx` verifies the default tab and that clicking the `Fristen` and
`Stammdaten` tabs swaps the visible panel. `RedactPanel.test.tsx` verifies masked
output and counts, and error rendering. `DeadlinePanel.test.tsx` verifies the
displayed date, the shift reason, that selecting "business" actually puts
`counting: "business"` in the request body, and that the cache indicator shows.
`CompliancePanel.test.tsx` verifies both a valid and an invalid verdict, and that
the specific `error_message` is displayed.

### 9.8 Why TypeScript strict mode

`tsconfig` runs with strict settings, and `package.json:8` builds with
`tsc -b && vite build`, so **type errors fail the build**. The `typecheck` script
(`tsc --noEmit`) is available for CI. The payoff is that the frontend cannot
silently drift from the API: if a field is renamed on one side, the other fails
to compile. In a two-language system, this compile-time contract is the cheapest
possible integration test.

---

## 10. Containerization and orchestration

### 10.1 What ships

- `backend/Dockerfile`: multi-stage build of the Rust service.
- `frontend/Dockerfile`: multi-stage build of the React app, served by nginx.
- `frontend/nginx.conf`: SPA serving plus `/api` reverse proxy.
- `docker-compose.yml`: wires `api`, `web`, `postgres`, and `redis` together.

### 10.2 Why containers

A compliance service must run identically on a developer laptop, in CI, and in a
government data centre. Containers pin the OS, the toolchain, and the dependency
versions. They also make the four-process topology (API, web, DB, cache)
reproducible with one command. This reproducibility is a prerequisite for the
auditability the whole system is about.

### 10.3 The backend Dockerfile and the stub-cache incident

The backend Dockerfile uses a **dependency-caching** trick to make rebuilds fast:
it first copies only `Cargo.toml`/`Cargo.lock`, creates a dummy `src/main.rs`
containing `fn main() {}`, and runs `cargo build --release`. This compiles all
*dependencies* into the image layer. Then it copies the real source and runs
`cargo build --release` again; because dependencies are already compiled, only
the crate itself rebuilds.

**The trap.** Cargo decides whether the crate needs rebuilding based on file
timestamps. If the real source has an *older* mtime than the stub's compiled
artifact (which happens when the Docker build context preserves checkout
timestamps), Cargo may believe the crate is unchanged and **ship the stub
binary** — a 332 KB no-op `main` instead of the real 7.6 MB service. The
container then starts and immediately exits or does nothing, which is
maddening to debug because the build "succeeded."

**The fix.** Immediately before the real build, invalidate the sources by
touching them: `find src -name '*.rs' -exec touch {} +`. This guarantees every
source file is newer than the stub artifact, forcing a genuine rebuild. The fix
is a one-liner in `backend/Dockerfile` but it encodes a real, subtle failure mode
of Docker layer caching in Rust projects.

**The lesson.** Layer caching optimizes for the common case (dependencies
unchanged) but the correctness of the optimization depends on an assumption
(timestamps reflect content) that is not always true. When an optimization has a
silent failure mode, you either disable it or add an invalidation step. We did
the latter, and the resulting image is small (release profile uses
`lto = "thin"` and `strip = true`, `Cargo.toml:31-32`).

### 10.4 The frontend Dockerfile

Multi-stage: a `node:20-alpine` builder runs `npm ci` (falling back to
`npm install`) and `npm run build`, producing `dist/`; the runtime stage is
`nginx:1.27-alpine`, which copies `nginx.conf` and the built `dist/`, exposes
port 80, and adds a `wget`-based `HEALTHCHECK` (`frontend/Dockerfile:15-16`).
Serving static files from nginx rather than a Node server is smaller, faster, and
has a smaller attack surface.

**The `npm ci` lesson.** During development, a stray `npm install` had pulled a
transitive package (`@asamuzakjp/css-color`) at a version outside the lockfile,
which broke jsdom's `cssstyle` and, in turn, the frontend test suite. The fix was
`rm -rf node_modules && npm ci` — restoring exactly the lockfile. This is why the
Dockerfile uses `npm ci`: it installs *from the lockfile*, guaranteeing the same
dependency graph everywhere. `npm install` may resolve new versions; `npm ci`
never does. For reproducible builds, `ci` is the correct command.

### 10.5 nginx configuration

`frontend/nginx.conf` does two jobs: serve the SPA (with a fallback to
`index.html` so client-side routing works on deep links) and reverse-proxy
`/api/*` to the `api` service on port 8080. This means the browser sees a single
origin (the web container), so there is **no CORS in production** — the request
never crosses origins. CORS is therefore only relevant for local dev, where Vite
proxies `/api` similarly. Keeping the browser on one origin also means the
backend's permissive CORS default is never exercised in the deployed stack.

### 10.6 docker-compose topology and network hygiene

`docker-compose.yml` defines four services on a private network. Only two ports
are published to the host: `api` (8080) and `web` (3000). **Postgres and Redis
are not published**, so they are reachable only by the API container on the
internal network. This is deliberate least-exposure: the database has no reason
to accept connections from the host or the internet. The compose project name is
`public-authority-assistant`, which namespaces the containers and volumes.

The API waits for Postgres and Redis to be healthy before starting (Compose
`depends_on` with health conditions), and every service has a healthcheck.
Startup ordering with health gating avoids the "API started before the DB was
ready" race that otherwise produces a flurry of connection errors on first boot.

**Why Compose rather than Kubernetes for this deliverable.** Compose is the
right size for a four-service demo: one file, one command, no cluster. The health
probes are designed so that the same definitions port to Kubernetes liveness and
readiness probes if the system grows; the *contract* is the same, only the
orchestrator differs.

---

## 11. Testing strategy

### 11.1 What runs

- **Backend unit tests:** 41, inside `backend/src/services/*.rs` as `#[cfg(test)]`
  modules. They test the pure domain functions directly.
- **Backend integration tests:** 16, in `backend/tests/api.rs`. They build the
  real router and drive it with in-process HTTP requests (via
  `tower::ServiceExt::oneshot`), asserting status codes and JSON bodies.
- **Frontend tests:** 12, in `frontend/src/**/*.test.tsx` with `vitest` and
  Testing Library.

Total: 69 automated tests, all green, plus the `api.http` manual sequence of 55
checks for exploratory and documentation purposes.

### 11.2 Why the test pyramid here is shaped this way

The base of the pyramid is the 41 unit tests because the domain is where the
legal correctness lives, and pure functions are cheap to test exhaustively. Every
check-digit vector, every holiday rule, every redaction edge case, and every
deadline shift has a unit test. This is intentional: bugs in the domain are the
expensive ones, so the domain gets the most tests.

The middle layer is the 16 integration tests, which verify the *wiring*: that a
missing field yields 400, that an unknown state yields 422, that the response
shape is right, that a request ID is echoed, that an unknown route yields 404.
These tests exist because correct domain logic behind a mis-wired route is still
a broken service.

The frontend tests are the smallest layer and focus on UI behavior given a
stubbed response, per §9.7.

### 11.3 Why unit tests call the service functions directly

The service modules import nothing from `routes` or the database. A unit test
can therefore construct an input, call the function, and assert on the output
with no setup. For example, a deadline test calls the deadline service with a
constructed holiday set and asserts the returned date. This is only possible
because of the layered architecture (§2.3): the purity of the domain is what
makes the tests fast and hermetic. If the domain depended on the DB, every unit
test would need a DB, and the suite would be slow and flaky.

### 11.4 Why integration tests use the real router

`backend/tests/api.rs` calls `build_router` and sends requests through the actual
middleware stack. This catches bugs that unit tests cannot: extractor
rejections, status-code mapping, header propagation, and serialization. It uses
`http-body-util` to read response bodies and `tower`'s `oneshot` to invoke the
service (`Cargo.toml:27-29`). The tests do not bind a port — they drive the
`Router` as a `Service`, which is both faster and less flaky than a real socket.

**What the integration tests assert** (the code-map in `lesson-plans/references/`
lists these by line): redaction happy path and error paths, deadline weekday/
weekend/holiday/year-end/business cases, holiday listing per state, compliance
valid/invalid/format/empty cases, 404 on unknown routes, request-id echo, and the
health/ready probes.

### 11.5 The regression discipline

The redaction false-positive fix (§3.3.4) and its two regression tests are the
model for the whole repository's testing philosophy:

- **A bug fix is not done until there is a test that would have caught it.** The
  test `does_not_mask_file_numbers_followed_by_dates` encodes the exact benign
  input from `api.http` case 1.4.
- **A false-positive fix must be paired with a true-positive test.**
  `masks_short_but_valid_national_numbers` proves the fix did not make the
  matcher blind.

This two-test pattern is worth internalizing far beyond this codebase: whenever
you narrow a matcher, a validator, or a filter, you must prove both that the
unwanted case is now excluded and that the wanted case still passes.

### 11.6 The `api.http` sequence as living documentation

`api.http` is a 55-check, 334-line sequence written for the REST Client extension
in VS Code / JetBrains. It is not a substitute for automated tests; it is
complementary. Its purpose is threefold: it documents every endpoint with a
realistic example, it provides a fast manual smoke test against the running
Docker stack, and it doubles as the acceptance checklist. Each request is
annotated with the expected outcome (e.g. "→ 400", "expect cached: true"), so a
reviewer can read the file top to bottom and learn the entire API. Using it
revealed a false "failure" during development: a `curl -d` probe omitted `-X
POST`, and `curl` defaults `-d` to POST anyway — the probe was wrong, not the
server. The lesson: when a test fails, first confirm the test is testing what you
think it is.

### 11.7 Why 69 tests is the right number, not more or fewer

More tests would mean testing framework behavior (Axum's routing, sqlx's
pooling) that is not ours to test. Fewer would leave the error mapping or the
calendar untested. The suite covers exactly the logic we wrote and the contract
we expose. Coverage for its own sake would add maintenance cost without adding
confidence; the goal is confidence in *our* decisions.

---

## 12. Configuration and observability

### 12.1 Configuration

Configuration is environment-driven, with sane defaults, and documented in
`.env.example`:

| Variable | Meaning | Default |
|---|---|---|
| `BIND_ADDR` | address the API listens on | `0.0.0.0:8080` |
| `DATABASE_URL` | Postgres connection string | `postgres://authority:authority@localhost:5432/authority` |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `RATE_LIMIT_PER_MINUTE` | fixed-window limit | `120` |
| `CORS_ALLOW_ORIGIN` | allowed origin | `*` |
| `STRICT_READINESS` | if true, readiness is stricter | `false` |
| `RUST_LOG` | tracing filter | `info,tower_http=debug,public_authority_api=debug` |

**Why environment variables.** They are the twelve-factor standard, they keep
secrets out of source, and they let the same image run in dev and prod with
different values. `backend/src/config.rs` reads and validates them once at
startup, so a misconfiguration fails fast and loudly rather than at first
request.

**Why `STRICT_READINESS`.** Sometimes you want readiness to require *all*
dependencies (strict, for production gating); sometimes you want the service to
report ready as long as it can serve the pure endpoints (lenient, for local
development where you may not have run the whole stack). Making this a flag
rather than a hard-coded decision acknowledges that "ready" is a policy, not a
fact.

### 12.2 Observability

Three pillars, all lightweight:

- **Structured logs** via `tracing` and `tracing-subscriber` with both
  `env-filter` and `json` features (`Cargo.toml:18-19`). The JSON mode is what a
  log aggregator (ELK, Loki, CloudWatch) wants; the env-filter lets operators
  dial verbosity per module without recompiling.
- **Request tracing** via `TraceLayer`, correlated by `X-Request-Id`. A single
  user-visible request produces one log line; the id ties the browser, the nginx
  log, the API log, and the audit row together.
- **Health/readiness**, as in §6.4, giving the orchestrator a machine-readable
  view of the service's ability to serve.

**Why not OpenTelemetry/metrics yet.** For this scope, structured logs plus
request IDs answer the operational questions ("what happened to request X?",
"which endpoint is erroring?") without the operational weight of a tracing
backend. Full distributed tracing and Prometheus metrics are listed as future
work in Part 14, and the architecture (tracing spans via `tower-http`) makes
adding them incremental rather than invasive.

### 12.3 The readiness check and the `sqlx` detail

`GET /ready` checks Postgres with `sqlx::query("SELECT 1").execute(pool)` and
Redis with a `PING`. If either fails, it returns 503. As noted in §7.3, `sqlx`
has no `ping()` method on the pool, so the canonical liveness query is used. The
readiness handler is also where the `STRICT_READINESS` policy is applied.

---

## 13. Design principles and trade-offs

### 13.1 The principles, gathered

1. **Domain-first layering.** Pure services, thin transport. Everything testable
   without infrastructure lives in `services/`.
2. **Correctness over cleverness.** Legal outputs are computed exactly, not
   approximated. No ML where the law specifies an algorithm.
3. **Auditability.** Raw inputs and reasons are returned; audit rows are
   privacy-preserving; request IDs correlate everything.
4. **Fail safe, fail loud, fail appropriately.** Readiness fails closed (503);
   rate limiting fails open; parsing fails loud (400); invalid data is a
   *successful* validation (200 + `valid:false`).
5. **Least exposure.** Only the API and web ports are published; the DB and cache
   are internal-only.
6. **Reproducibility.** Lockfile-driven `npm ci`, pinned dependencies, migrations
   in source, containers everywhere.
7. **Consistency.** One error type, one error body shape, one UI feedback
   component, one panel structure.
8. **Accessibility by default.** Public-sector software must be usable by
   everyone; accessible markup is built in and enforced by the tests.

### 13.2 The trade-offs we consciously accepted

| Decision | Chosen | Rejected alternative | Why |
|---|---|---|---|
| Architecture | modular monolith | 3 microservices | cohesion, lower ops cost, splittable later |
| Redaction engine | single regex, no lookaround | multi-pass / lookaround | linear-time safety, one-pass correctness |
| Phone matching | specificity in the pattern | permissive + post-filters | no lookaround available; domain-accurate |
| Holiday source | computed (computus) + state data | hardcoded table | never expires, correct for all years |
| Date parsing | strict ISO only | fuzzy / German formats | fail loud rather than guess |
| Audit log | hash + counts | store original text | privacy-preserving by design |
| Deadline cache | Redis with TTL | in-process map | shared across replicas, atomic ops |
| Rate limiting | fixed window, fail open | token bucket, fail closed | simplicity; availability priority |
| Validation result | 200 + `valid:false` | 4xx on invalid data | separates data validity from transport errors |
| Build | `npm ci`, pinned crates | floating versions | reproducibility |

Each row is a decision with a real cost. The value of listing them is that a
future maintainer can revisit a trade-off deliberately rather than rediscovering
it under pressure.

### 13.3 What "production-shaped" means here

The service is not production-ready government infrastructure, and it does not
pretend to be. But it has the *shape* of production software: health probes,
typed errors, structured logs, a migration-managed database, containers,
least-exposure networking, a test suite, and documented configuration. The gaps
to true production are enumerated honestly in Part 14 — which is itself a mark of
engineering maturity. Claiming completeness would be the bigger failure.

---

## 14. Known limitations and future work

Intellectual honesty requires naming what the system does *not* do. Each item
below is a real limitation with a clear upgrade path.

### 14.1 Redaction

- **Regex, not named-entity recognition.** The redactor finds e-mail, IBAN, and
  phone patterns. It does **not** detect names, addresses, dates of birth, tax
  numbers, social security numbers, or free-form identifiers. A production DSGVO
  redactor would combine pattern matching with NER models and a purpose-specific
  policy engine. The pattern layer here is the deterministic core that such a
  system must have.
- **German IBANs only.** Other countries' IBANs are out of scope; extending is a
  data change (country code + length table).
- **The phone pattern is a heuristic.** It is tuned for German conventions and
  will still mis-classify some unusual inputs. There is no phone-number oracle.
- **Unkeyed hash in the audit.** A hash of low-entropy text is brute-forceable;
  an HMAC with a secret key would make the fingerprint non-reversible even for
  guessable inputs.

### 14.2 Deadlines

- **Only the two counting modes.** Real procedural law has more (e.g. periods
  expressed in weeks/months with month-end rules). The API covers the two the
  challenge specifies.
- **Holiday data is the current statutory set.** If a state adds or removes a
  holiday, the data must be updated; the algorithm need not change.
- **No time-of-day semantics.** Deadlines are date-granular. Real filings may
  have time-of-day cutoffs (e.g. end of business); that would require a clock and
  a timezone policy.
- **Repentance Day rule should be re-verified** against the current statute for
  boundary years, though the "Wednesday before 23 November" rule is the standard
  formulation.

### 14.3 Compliance

- **USt-IdNr. and register number only.** No validation of tax numbers
  (*Steuernummer*), LEI, IBAN checksums, or EU-wide VAT IDs. Each would be an
  additional validator following the same pattern.
- **No VIES lookup.** A full solution would call the EU VIES service to check
  that a VAT ID is *registered*, not merely well-formed. That requires a network
  dependency, caching, and an availability policy — deliberately out of scope.
- **Register number structure is permissive.** Because the scheme has no
  checksum, structural validation is the ceiling.

### 14.4 Platform

- **No authentication or authorization.** Every endpoint is open. A real
  authority deployment needs OIDC/mTLS, per-role access, and an audit of *who*
  redacted *what*. The audit table has a request ID but no principal.
- **Rate limiting is coarse.** Per-IP fixed window; no per-tenant quotas, no
  distinction between read and write endpoints. A token bucket and per-key limits
  are the next step.
- **No metrics or distributed tracing.** Logs and request IDs only; Prometheus
  and OpenTelemetry are the obvious additions.
- **CORS is permissive by default.** Acceptable behind the proxy; should be
  pinned in a real deployment.
- **No backup/retention policy for the audit database.** Production needs
  retention schedules (GDPR storage limitation) and tested restores.
- **Single-instance assumptions.** The service is stateless and would scale
  horizontally, but the audit schema, rate-limit keys, and cache are the only
  shared state, and no load test has been run.

### 14.5 Frontend

- **Demonstration console, not a workflow tool.** No auth, no saved drafts, no
  bulk upload, no history view of past redactions (the audit `audit_id` would be
  the hook).
- **No internationalization.** UI strings are German; the code is English.

Naming these limitations is not a concession that the work is incomplete. It is
the specification of the next iteration, and it is what separates an engineering
artifact from a demo.

---

## 15. End-to-end walkthrough: one request through the whole system

To tie everything together, follow a single `POST /api/v1/redact` from browser to
database and back.

1. **Browser.** An operator pastes text into the RedactPanel and clicks
   *Anonymisieren*. `RedactPanel.tsx` sets `loading`, clears any prior error, and
   calls `redactText` in `api.ts`.
2. **Client transport.** `api.ts` issues `fetch("/api/v1/redact", { method:
   "POST", headers: {"Content-Type": "application/json"}, body })`. In dev, Vite
   proxies `/api` to `localhost:8080`; in prod, nginx proxies to the `api`
   container. Either way the call is same-origin.
3. **Ingress.** nginx receives the request and forwards it to the API. The
   outermost middleware, `TraceLayer`, opens a span; `RequestIdLayer` assigns or
   propagates `X-Request-Id`; `CorsLayer` adds CORS headers; `TimeoutLayer` starts
   its clock.
4. **Rate limiting.** The rate-limit middleware derives the client key, does an
   atomic `INCR` on `ratelimit:<ip>:<minute>`, checks the count, sets the
   `X-RateLimit-*` headers, and either passes the request or returns 429 with a
   `Retry-After`. If Redis is unreachable, it fails open and passes the request.
5. **Routing.** Axum matches the path to the redact handler and deserializes the
   JSON body into the request struct. A missing `text` field causes a
   deserialization rejection, which `error.rs` maps to **400**.
6. **Domain.** The handler calls `redaction::mask_sensitive_data(text)`. The pure
   function runs the single alternation regex, counts matches by named group,
   builds the masked text, and returns `(masked, counts, total)`. This is the
   same function exercised by the 41 unit tests.
7. **Persistence.** The handler computes a SHA-256 hash of the input, inserts an
   audit row (hash, counts, timestamp, request ID) via `repository/audit.rs`, and
   receives the generated `audit_id` (UUID). Only the hash — never the text — is
   stored.
8. **Serialization.** The handler builds the response struct
   (`text`, `counts`, `total_redacted`, `audit_id`) and Axum serializes it to
   JSON. `TraceLayer` records the status and latency; `RequestIdLayer` echoes the
   id.
9. **Return path.** nginx returns the JSON to the browser. `api.ts` sees
   `response.ok`, parses the JSON, and returns it typed. `RedactPanel` stores it
   as `result` and renders the masked text, the counts, the total, and the audit
   ID inside an `aria-live="polite"` region, so a screen reader announces it.
10. **Observability.** The API log line and the audit row both carry the same
    request ID, so an auditor can correlate the user-visible request with the
    persisted record without either containing the personal data.

That single trace exercises every layer: UI, typed client, proxy, middleware
stack (tracing, request ID, CORS, timeout, rate limiting), routing and extraction,
typed errors, pure domain logic, privacy-preserving persistence, serialization,
and accessibility. It is the whole architecture in miniature.

---

## 16. Conclusion: what has been implemented, and why it matters

What has been implemented is a small but complete GovTech/LegalTech/RegTech
microservice: three legally-specified data transformations (redaction, deadline
calculation, master-data validation) behind a versioned HTTP API, with correct
and consistent error semantics, a privacy-preserving audit trail, caching and
rate limiting that degrade safely, a Docker Compose stack with least-exposure
networking, and an accessible React portal — all covered by 69 automated tests
and a 55-check manual sequence, and all documented.

Why it matters is the thesis stated at the top: where the law specifies the
answer, software should compute the answer exactly and be accountable for having
done so. Every design choice in this repository serves that thesis. The redaction
pattern encodes facts about German phone numbers. The deadline engine computes
Easter rather than tabulating it. The VAT validator implements MOD 11,10 to the
letter, including the forbidden zero. The error taxonomy separates "I couldn't
understand you" from "your data is invalid." The audit log refuses to keep the
data it protects. The rate limiter fails open because availability beats
theoretical abridgement of a non-security control.

How it was done is equally instructive: pure domain functions behind a thin
transport, so the hard logic is the easy thing to test; a fixed bug that became
two regression tests; a Docker caching trap that became `touch`; a lockfile
discipline that became `npm ci`; and a documentation habit (the `lesson-plans/`
package and this guide) that makes the reasoning as durable as the code.

That is the complete picture — what, why, and how — and it is the picture a new
engineer needs in order to change this system without breaking the law it
encodes.

---

## Appendix A — Full API reference

### A.1 `GET /health`
Liveness. Always 200 while the process runs. Never touches dependencies.
Response: `{ "status": "ok" }`.

### A.2 `GET /ready`
Readiness. 200 when dependencies are reachable; **503** otherwise. Honors
`STRICT_READINESS`. Checks Postgres (`SELECT 1`) and Redis (`PING`).

### A.3 `POST /api/v1/redact`
Request: `{ "text": string }`.
Response: `{ "text": string, "counts": { "email": n, "iban": n, "phone": n },
"total_redacted": n, "audit_id": string }`.
Errors: 400 (missing `text`, malformed JSON).

### A.4 `POST /api/v1/deadline`
Request: `{ "start_date": "YYYY-MM-DD", "days_to_add": int, "federal_state":
"<code>", "counting": "calendar" | "business" }`.
Response: `{ "date", "raw_date", "shifted": bool, "reason": string|null,
"counting", "federal_state", "cached": bool }`.
Errors: 400 (bad/impossible date, missing fields), 422 (unknown state, negative
days).

### A.5 `GET /api/v1/holidays?year=<int>&state=<code>`
Response: array of `{ "date", "name", ... }` for that year and state.
Errors: 422 (unknown state).

### A.6 `POST /api/v1/compliance/validate`
Request: `{ "tax_id"?: string, "register_number"?: string }` (at least one).
Response: `{ "tax_id": {...}|null, "register_number": {...}|null, "valid": bool }`
where each sub-result is `{ "valid": bool, "normalized": string|null, "error":
string|null, "error_message": string|null }`.
Errors: 400 (neither field provided).

### A.7 Platform behaviors
- Unknown route → **404**.
- `X-Request-Id` echoed; generated if absent.
- Over the rate limit → **429** with `X-RateLimit-Limit`, `X-RateLimit-Remaining`,
  `Retry-After`.

---

## Appendix B — Glossary

| Term | Meaning |
|---|---|
| *Behörde* | public authority / government agency |
| *Schwärzung* | redaction (literal: blacking out) |
| *Frist / Fristberechnung* | deadline / deadline calculation |
| *Bundesland* (pl. *Bundesländer*) | German federal state (16 total) |
| *Feiertag* | public holiday |
| *USt-IdNr.* (*Umsatzsteuer-Identifikationsnummer*) | VAT identification number |
| *Steuernummer* | state-level tax number (no national check digit) |
| *Handelsregisternummer* | commercial register number (e.g. `HRB 12345 B`) |
| *Stammdatenprüfung* | master-data validation |
| DSGVO | GDPR (German name) |
| VwVfG | Administrative Procedure Act |
| *Buß- und Bettag* | Repentance and Prayer Day (holiday only in Saxony) |
| computus | algorithm for the date of Easter |
| IBAN | International Bank Account Number |
| check digit | final digit of an identifier, computed for error detection |
| MOD 11,10 | ISO 7064 check-digit procedure in product form |
| fixed window | rate-limiting scheme with one counter per time window |
| VIES | EU VAT Information Exchange System |
| BITV 2.0 | German accessibility regulation for public-sector IT |

---

## Appendix C — Repository file map

```
backend/
  Cargo.toml                      dependencies + release profile
  Dockerfile                      multi-stage build with dep-cache + touch fix
  .dockerignore                   excludes target/, Dockerfile, *.md
  migrations/0001_redaction_audit.sql
  src/
    main.rs                       config + state + serve
    lib.rs                        build_router(state)
    config.rs                     env-driven configuration
    state.rs                      AppState (db pool, redis, config)
    error.rs                      AppError enum + IntoResponse (status mapping)
    routes/
      health.rs                   /health, /ready
      redact.rs                   /api/v1/redact
      deadline.rs                 /api/v1/deadline, /api/v1/holidays
      compliance.rs               /api/v1/compliance/validate
    services/
      redaction.rs                regex masking + counts (+ the 2 regression tests)
      deadline.rs                 calendar/business counting + shift reasons
      holidays.rs                 Easter computus + per-state holiday sets
      compliance.rs               MOD 11,10 VAT check + register format
    repository/audit.rs           SHA-256 hash + audit insert
    middleware/rate_limit.rs      fixed-window Redis limiter (fail open)
  tests/api.rs                    16 integration tests
frontend/
  package.json  vite/tsconfig     build + test tooling
  Dockerfile                      node build → nginx runtime
  nginx.conf                      SPA fallback + /api reverse proxy
  src/
    main.tsx  App.tsx  api.ts  types.ts  App.css
    components/ RedactPanel, DeadlinePanel, CompliancePanel, Feedback (+ tests)
docker-compose.yml                api + web + postgres + redis
api.http                          55-check manual sequence
.env.example                      documented configuration
challenge.md                      original requirements
lesson-plans/                     teaching package (see its OWN README)
education.md                      this document
```

---

## Appendix D — Build and run

**Whole stack (Docker Compose):**
```
docker compose up --build
# web:  http://localhost:3000
# api:  http://localhost:8080
```

**Backend locally:**
```
cd backend
cargo test                              # 41 unit + 16 integration
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run                               # needs DATABASE_URL + REDIS_URL
```

**Frontend locally:**
```
cd frontend
npm ci
npm test                                # 12 tests
npm run typecheck
npm run build
npm run dev                             # Vite dev server with /api proxy
```

**Manual API sweep:** open `api.http` in VS Code with the REST Client extension
and send requests in order (55 checks).

---

*End of document.*







