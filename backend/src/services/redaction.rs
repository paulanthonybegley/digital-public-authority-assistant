//! GDPR-compliant redaction of personal data in free text.
//!
//! A single combined pattern scans the text in one pass so that matches never
//! interfere with each other (an IBAN cannot be partially re-matched as a
//! phone number after redaction, etc.).

use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

static SENSITIVE: LazyLock<Regex> = LazyLock::new(|| {
    // Order matters: email | IBAN | phone. First alternative wins at a position.
    //
    // Phone:
    //  - international: +49 / 0049 forms;
    //  - national: `0` + area code starting 1-9 (German national numbers never
    //    begin with 00 - that is the international prefix) + separator +
    //    subscriber of 5+ digits (rejects date tails like `2026/0042 2026`).
    // Rust's regex crate has no lookaround, so these constraints live in the
    // pattern itself rather than in lookbehind assertions.
    let pattern = concat!(
        r"(?P<email>[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,})",
        r"|(?P<iban>DE\d{2}\s?(?:\d{4}\s?){4}\d{2})",
        r"|(?P<phone>(?:\+|00)49[\s\-]?\(?\d{1,4}\)?[\s\-]?\d{5,}(?:[\s\-/]\d{2,4})?|\b0[1-9]\d{0,3}[\s\-/]\d{5,})",
    );
    Regex::new(pattern).expect("static redaction pattern is valid")
});

pub const EMAIL_PLACEHOLDER: &str = "[EMAIL_REDACTED]";
pub const IBAN_PLACEHOLDER: &str = "[IBAN_REDACTED]";
pub const PHONE_PLACEHOLDER: &str = "[PHONE_REDACTED]";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct RedactionCounts {
    pub email: usize,
    pub iban: usize,
    pub phone: usize,
}

impl RedactionCounts {
    pub fn total(&self) -> usize {
        self.email + self.iban + self.phone
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RedactionResult {
    pub text: String,
    pub counts: RedactionCounts,
}

/// Mask e-mail addresses, German IBANs and German phone numbers.
pub fn mask_sensitive_data(text: &str) -> RedactionResult {
    let mut counts = RedactionCounts::default();
    let mut out = String::with_capacity(text.len());
    let mut last = 0usize;

    for caps in SENSITIVE.captures_iter(text) {
        let m = caps.get(0).expect("match 0 always present");
        out.push_str(&text[last..m.start()]);
        if caps.name("email").is_some() {
            counts.email += 1;
            out.push_str(EMAIL_PLACEHOLDER);
        } else if caps.name("iban").is_some() {
            counts.iban += 1;
            out.push_str(IBAN_PLACEHOLDER);
        } else if caps.name("phone").is_some() {
            counts.phone += 1;
            out.push_str(PHONE_PLACEHOLDER);
        } else {
            // Unreachable with the current pattern; keep the text untouched.
            out.push_str(m.as_str());
        }
        last = m.end();
    }
    out.push_str(&text[last..]);

    RedactionResult { text: out, counts }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_email() {
        let r = mask_sensitive_data("Contact: max.mustermann@example.de for details.");
        assert_eq!(r.text, "Contact: [EMAIL_REDACTED] for details.");
        assert_eq!(r.counts.email, 1);
        assert_eq!(r.counts.total(), 1);
    }

    #[test]
    fn masks_email_with_plus_and_subdomain() {
        let r = mask_sensitive_data("a.b+tag@mail.example.co.uk");
        assert_eq!(r.text, "[EMAIL_REDACTED]");
        assert_eq!(r.counts.email, 1);
    }

    #[test]
    fn masks_iban_with_and_without_spaces() {
        let spaced = mask_sensitive_data("IBAN DE89 3704 0044 0532 0130 00 bitte prüfen.");
        assert_eq!(spaced.text, "IBAN [IBAN_REDACTED] bitte prüfen.");
        assert_eq!(spaced.counts.iban, 1);

        let compact = mask_sensitive_data("Zahlung per DE89370400440532013000 erfolgt.");
        assert_eq!(compact.text, "Zahlung per [IBAN_REDACTED] erfolgt.");
        assert_eq!(compact.counts.iban, 1);
    }

    #[test]
    fn masks_international_and_national_phone_numbers() {
        let cases = [
            ("Tel +49 30 12345678", "Tel [PHONE_REDACTED]"),
            ("Tel: +493012345678", "Tel: [PHONE_REDACTED]"),
            ("Rufnummer 0049 30 12345678", "Rufnummer [PHONE_REDACTED]"),
            ("Kontakt 030/1234567", "Kontakt [PHONE_REDACTED]"),
            ("Kontakt 030-12345678", "Kontakt [PHONE_REDACTED]"),
            ("Mobil 0170 1234567", "Mobil [PHONE_REDACTED]"),
        ];
        for (input, expected) in cases {
            let r = mask_sensitive_data(input);
            assert_eq!(r.text, expected, "input: {input}");
            assert_eq!(r.counts.phone, 1, "input: {input}");
        }
    }

    #[test]
    fn masks_mixed_document_and_counts_per_category() {
        let doc = "Sehr geehrte Frau Müller,\n\
             Ihre Bewerbung unter m.mueller@web.de wurde geprueft.\n\
             Die Ueberweisung (DE02120300000000202051) ist eingegangen.\n\
             Bitte rufen Sie unter 030/5551234 an. Freundliche Gruesse, Referat B4.";
        let r = mask_sensitive_data(doc);
        assert_eq!(r.counts.email, 1);
        assert_eq!(r.counts.iban, 1);
        assert_eq!(r.counts.phone, 1);
        assert_eq!(r.counts.total(), 3);
        assert!(r.text.contains("[EMAIL_REDACTED]"));
        assert!(r.text.contains("[IBAN_REDACTED]"));
        assert!(r.text.contains("[PHONE_REDACTED]"));
        // Non-sensitive content is preserved verbatim.
        assert!(r.text.contains("Sehr geehrte Frau Müller"));
        assert!(r.text.contains("Referat B4"));
    }

    #[test]
    fn adjacent_entities_do_not_bleed_into_each_other() {
        let r = mask_sensitive_data("a@b.de DE89370400440532013000 030/1234567");
        assert_eq!(r.text, "[EMAIL_REDACTED] [IBAN_REDACTED] [PHONE_REDACTED]");
        assert_eq!(r.counts.total(), 3);
    }

    #[test]
    fn empty_and_unrelated_text_is_unchanged() {
        for input in [
            "",
            "   ",
            "Aktenzeichen AZ-2026/0042 ohne Daten.",
            "nur ein Satz.",
        ] {
            let r = mask_sensitive_data(input);
            assert_eq!(r.text, input);
            assert_eq!(r.counts.total(), 0);
        }
    }

    #[test]
    fn unicode_text_is_preserved() {
        let r = mask_sensitive_data("Grüße aus München – e-mail an test@example.org.");
        assert_eq!(r.text, "Grüße aus München – e-mail an [EMAIL_REDACTED].");
        assert_eq!(r.counts.email, 1);
    }

    #[test]
    fn does_not_mask_dates_or_iban_like_fragments() {
        // A date must not be mistaken for a phone number.
        let r = mask_sensitive_data("Datum 2026-05-01, Betrag 1.234,56 EUR");
        assert_eq!(r.counts.total(), 0);
        assert_eq!(r.text, "Datum 2026-05-01, Betrag 1.234,56 EUR");
    }

    #[test]
    fn does_not_mask_file_numbers_followed_by_dates() {
        // Regression: `2026/0042 2026-05-01` once matched as `0042 2026`.
        // Two rules prevent this: national numbers never start with `00`
        // (international prefix), and the subscriber needs >= 5 digits.
        for input in [
            "AZ-2026/0042 2026-05-01 1.234,56 EUR",
            "Vorgang 2026/0123 2026-05-01",
            "Aktenzeichen 0042 2026",
        ] {
            let r = mask_sensitive_data(input);
            assert_eq!(
                r.counts.total(),
                0,
                "unexpected redaction in {input:?}: {:?}",
                r.text
            );
            assert_eq!(r.text, input, "text altered for {input:?}");
        }
    }

    #[test]
    fn masks_short_but_valid_national_numbers() {
        // Real German numbers with a 4-digit subscriber after a 2-digit area
        // code must still be caught (Berlin 030, Munich 089, ...).
        for input in ["030 12345", "089/123456", "040-1234567"] {
            let r = mask_sensitive_data(input);
            assert_eq!(
                r.counts.phone, 1,
                "expected phone match in {input:?}: {:?}",
                r.text
            );
        }
    }

    #[test]
    fn multiple_emails_all_masked() {
        let r = mask_sensitive_data("an: a@x.de, b@y.org; cc: c@z.eu");
        assert_eq!(r.counts.email, 3);
        assert_eq!(
            r.text,
            "an: [EMAIL_REDACTED], [EMAIL_REDACTED]; cc: [EMAIL_REDACTED]"
        );
    }
}
