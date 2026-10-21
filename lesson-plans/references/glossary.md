# Reference — Glossary

Domain (German administrative law / compliance) and technical vocabulary used in
this course. Terms are grouped so a teacher can preview just the ones a session
needs.

---

## Domain terms (German public administration)

**DSGVO / GDPR** — Datenschutz-Grundverordnung, the EU General Data Protection
Regulation. Governs how personal data may be processed. Key principle used here:
**data minimization** (Art. 5(1)(c)) — collect/keep only what is necessary.

**Personenbezogene Daten / PII** — personal data; information that identifies a
person (email, IBAN, phone number, name). The redactor targets three of these.

**Schwärzung** — "redaction" / blacking out. The German UI tab label for the
redaction feature.

**VwVfG § 32** — Verwaltungsverfahrensgesetz (Administrative Procedure Act),
§ 32. The rule that a deadline ending on a Saturday, Sunday, or public holiday
ends on the next business day. (`deadline.rs:1` cites it.)

**ZPO § 193** — Zivilprozessordnung (Civil Procedure Code), § 193. The analogous
rule in civil proceedings.

**Feiertag / gesetzlicher Feiertag** — public holiday, established by each
*Bundesland* (federal state). Some are nationwide, some state-specific
(`holidays.rs`).

**Bundesland / federal state** — one of Germany's 16 states, coded by ISO
3166-2 (`BY`, `BE`, `NW`, …). Holiday observance differs per state.

**Beweglicher Feiertag** — "moving holiday", e.g. Easter and everything defined
relative to it. Must be computed, not stored.

**USt-IdNr. / VAT ID** — Umsatzsteuer-Identifikationsnummer. German form:
`DE` + 9 digits, with a mod-11 check digit (`compliance.rs`).

**Prüfziffer / check digit** — a redundant digit computed from the others so
typos can be detected. German VAT IDs use the mod-11 *product method*.

**Handelsregisternummer / register number** — commercial register entry such as
`HRB 12345 B` (HRB = Handelsregister Abteilung B, i.e. GmbHs/AGs). Sequential,
no check digit.

**Handelsregister** — the German commercial register; entries are assigned
sequential numbers.

**Aktenzeichen (Az.)** — case/file reference number, e.g. `AZ-2026/0042`. Not
personal data; the redactor must **not** touch it.

---

## Technical terms

**Microservice** — a small, independently deployable service with a focused
responsibility. This backend is one such service.

**Pure function** — a function whose output depends only on its inputs and which
has no side effects. The three services are pure; this is what makes them easy
to unit-test.

**Regex / regular expression** — a pattern language for matching text. The
redactor combines three patterns into one (`redaction.rs:21-25`).

**Lookaround / lookbehind** — regex constructs (`(?=…)`, `(?<=…)`) that match
without consuming. **Rust's `regex` crate does not support them**, so constraints
must be encoded in the pattern itself (`redaction.rs:19-20`).

**False positive / false negative** — in matching: a false positive redacts
something harmless; a false negative leaks something sensitive. Here a false
negative is considered worse (privacy), but false positives corrupt documents,
so both are tested (`redaction.rs:184,205`).

**Single-pass automaton** — scanning the text once with a combined pattern, vs.
multiple sequential replacements that can interfere (`redaction.rs:2-4`).

**Placeholder / mask** — the constant string substituted for a match, e.g.
`[EMAIL_REDACTED]`.

**Named capture group** — a regex group with a label (`(?P<email>…)`) so the
code can tell which category matched (`redaction.rs:58-75`).

**Calendar day vs. business day** — calendar counting adds every day; business
counting skips Saturdays, Sundays, and public holidays (`deadline.rs:10-36`).

**Computus** — the calculation of the date of Easter. This repo uses the
*anonymous Gregorian computus* (`holidays.rs:78-94`), so no moving dates are
hardcoded.

**Leap year** — a year with 366 days (divisible by 4, except centuries not
divisible by 400). Tested at `deadline.rs:227`.

**Mod-11 / mod-11 product method** — a check-digit algorithm: weight the digits
(`2,7,6,5,4,3,2,1`), take the digit sum of each product, sum, then
`check = (10 - sum % 10) % 10`. The check digit `0` is forbidden
(`compliance.rs:47-63`).

**Normalization** — cleaning input into a canonical form (trim, uppercase, strip
separators) *before* validating (`compliance.rs:36-42`).

**Modulo / `%`** — remainder of integer division; used throughout the check
digit.

**Typed error** — an error represented as an enum variant rather than a string,
so callers can react programmatically (`error.rs`, `TaxIdError`,
`DeadlineError`).

**`IntoResponse`** — the Axum trait that converts a value into an HTTP response;
`AppError` implements it to map variants to status codes (`error.rs:41-58`).

**Middleware / layer** — code that wraps request handling to add cross-cutting
behaviour (rate limiting, timeouts, CORS, tracing, request ids)
(`lib.rs:28-42`).

**State (`AppState`)** — shared, cloneable application data (config + optional
DB/Redis handles) passed to handlers (`state.rs`).

**Fixed-window rate limiting** — counting requests per client within a fixed
time window (here 60s) and rejecting past the limit
(`rate_limit.rs:12,52`).

**Fail open** — on dependency failure, continue serving instead of erroring. The
rate limiter and deadline cache do this (`rate_limit.rs:76-79`).

**Readiness vs. liveness** — liveness (`/health`) = the process is up; readiness
(`/ready`) = dependencies are reachable (`health.rs`).

**SHA-256** — a cryptographic hash. The audit stores the hash of the original
payload, not the payload, so the PII is not retained (`audit.rs:24-26`).

**Data minimization** — the GDPR principle of not keeping more personal data
than necessary; the audit table is the worked example.

**Migration** — a versioned SQL change applied at startup
(`migrations/0001_redaction_audit.sql`).

**Multi-stage build** — a Dockerfile with separate build and runtime stages to
ship a small runtime image (`backend/Dockerfile`).

**Cache stampede / TTL** — not implemented concerns, but useful discussion
topics; the deadline cache uses a 3600s TTL (`deadline.rs:46`).

---

## Frontend terms

**SPA** — single-page application; the React portal.

**Reverse proxy** — nginx forwards `/api/*` to the backend inside the Docker
network (`frontend/nginx.conf:13-19`).

**Typed client** — `frontend/src/api.ts` mirrors the backend JSON shapes from
`frontend/src/types.ts`, so TypeScript catches contract drift.
