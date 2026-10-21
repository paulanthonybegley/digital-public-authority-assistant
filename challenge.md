Here is a tailored 90-minute coding challenge draft designed for the Judiciary / Public Administration / Business sector (GovTech / LegalTech / Compliance).

The focus is not on abstract algorithms, but on typical practical problems from these sectors: GDPR-compliant data anonymization, deadline calculation, or document verification.

Coding Challenge: "The Digital Public Authority Assistant"
Duration: 90 minutes

Target Audience: Software Developers (Backend / Fullstack: e.g., Python, Node.js, Java, or C#)

Context: A state authority / legal service provider requires a microservice that automatically checks incoming digital briefs and corporate data prior to further processing, calculates deadlines, and protects sensitive data.

Scenario & Task Description
You are developing a prototype for a citizen and business portal. The system receives structured JSON data representing applications, lawsuits, or corporate filings.

In 90 minutes, you need to implement three core components (User Stories):

GDPR-Compliant Redaction (Data Masking): Personally identifiable information (names, email addresses, IBANs) in text fields must be redacted using regular expressions or masking logic.

Statutory Deadline Calculation (Legal Date Logic): Taking weekends and public holidays into account, the official deadline for an administrative decision must be determined.

Validation of Master Company Data (Compliance Check): Verification of registration numbers and VAT IDs according to specified check-digit algorithms.

Timeline (Timeboxing)
00:00 – 00:10: Setup, reading requirements & data structures.

00:10 – 00:35: Task 1 – Text Anonymization (GDPR).

00:35 – 01:05: Task 2 – Deadline calculation taking public holidays into account.

01:05 – 01:25: Task 3 – Validation of corporate data (check digits).

01:25 – 01:30: Final code check, tests, and brief concluding comments.

Detailed Tasks
Task 1: The Redaction Automaton (GDPR)
Problem: Documents and free text in administrative proceedings contain sensitive plain text data that must be protected before inspection of files is granted.

Task: Write a function mask_sensitive_data(text: str) -> str that masks the following patterns in the provided text:

Email addresses: Replace with [EMAIL_REDACTED] (or ***).

IBANs (DE standard): Replace IBANs in the format DE\d{2}\s?(\d{4}\s?){4}\d{2} with [IBAN_REDACTED].

Phone numbers: Anonymize common German formats (e.g., +49 ... or 030/...).

Task 2: Official Deadline Calculation
Problem: Under German law (e.g., VwVfG / ZPO), deadlines end under specific conditions. If the deadline falls on a Saturday, Sunday, or a statutory public holiday, the end date shifts to the next business day.

Task: Write a function calculate_deadline(start_date: str, days_to_add: int, federal_state: str) -> str:

Add days_to_add (e.g., 1 month or 2 weeks – ideal for the challenge: business days/calendar days from start date) to the start_date (Format: YYYY-MM-DD).

Check whether the calculated date falls on a weekend.

Bonus/Optional: Include at least three fixed public holidays of the state (e.g., New Year's Day, Labour Day, October 3rd). If the date falls on one of these, shift it to the next business day.

Task 3: Compliance & Commercial Register Check
Problem: Companies submitting applications must provide valid identification criteria.

Task: Implement a validation function for a simplified Value Added Tax Identification Number (VAT ID) or a Commercial Register Number:

A German VAT ID starts with the country code DE followed by 9 digits (e.g., DE123456789).

Write a function validate_tax_id(tax_id: str) -> bool that verifies the format and optionally applies a simplified Modulo-11 algorithm to the digits.

Evaluation Criteria (What the Interviewer / Evaluator Looks For)
Correctness & Robustness: Do unit tests work for edge cases (e.g., year-end transitions for deadlines, invalid formats)?

Code Quality & Readability: Meaningful variable names, clean structure, modularization (no monolithic spaghetti code).

Handling of Edge Cases: What happens with empty input, wrong date formats, or special characters?

Pragmatism: The entire German public holiday logic does not need to be covered in 90 minutes – what matters is whether intelligent placeholders or clean mock logic are used to achieve the goal on time.