//! Compliance checks for master company data:
//! German VAT identification numbers (USt-IdNr., mod-11 check digit)
//! and commercial register numbers (Handelsregisternummer, format check).

use serde::Serialize;

/// Why a VAT ID failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxIdError {
    Empty,
    InvalidFormat,
    InvalidCheckDigit,
}

impl TaxIdError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Empty => "tax ID must not be empty",
            Self::InvalidFormat => "German VAT ID must be 'DE' followed by exactly 9 digits",
            Self::InvalidCheckDigit => "mod-11 check digit verification failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaxIdReport {
    pub valid: bool,
    /// Normalized form (`DE123456789`), `null` when unparseable.
    pub normalized: Option<String>,
    pub error: Option<TaxIdError>,
    pub error_message: Option<String>,
}

/// Normalize a VAT ID: trim, uppercase, strip separators (`DE 123.456.789`).
fn normalize_tax_id(raw: &str) -> String {
    raw.trim()
        .to_ascii_uppercase()
        .chars()
        .filter(|c| !matches!(c, ' ' | '.' | '-' | '/'))
        .collect()
}

/// Official mod-11 check digit for the first 8 digits of a German VAT ID
/// (product method, § 150 AO): weights 2,7,6,5,4,3,2,1; digit sums of products;
/// check = (10 - sum % 10) % 10, and the check digit must not be 0.
pub fn compute_check_digit(first_eight: &str) -> Option<u32> {
    const WEIGHTS: [u32; 8] = [2, 7, 6, 5, 4, 3, 2, 1];
    if first_eight.len() != 8 || !first_eight.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let digits: Vec<u32> = first_eight.bytes().map(|b| u32::from(b - b'0')).collect();
    let sum: u32 = digits
        .iter()
        .zip(WEIGHTS)
        .map(|(d, w)| {
            let product = d * w;
            (product % 10) + (product / 10)
        })
        .sum();
    let check = (10 - (sum % 10)) % 10;
    (check != 0).then_some(check)
}

/// Validate a German VAT ID (format + mod-11 check digit).
pub fn validate_tax_id(raw: &str) -> TaxIdReport {
    let fail = |error: TaxIdError, normalized: Option<String>| TaxIdReport {
        valid: false,
        normalized,
        error: Some(error),
        error_message: Some(error.message().to_string()),
    };

    if raw.trim().is_empty() {
        return fail(TaxIdError::Empty, None);
    }
    let normalized = normalize_tax_id(raw);

    let Some(digits) = normalized.strip_prefix("DE") else {
        return fail(TaxIdError::InvalidFormat, Some(normalized));
    };
    if digits.len() != 9 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return fail(TaxIdError::InvalidFormat, Some(normalized));
    }

    let check_digit = digits[8..9].parse::<u32>().expect("digit slice parses");
    match compute_check_digit(&digits[..8]) {
        Some(expected) if expected == check_digit => TaxIdReport {
            valid: true,
            normalized: Some(normalized),
            error: None,
            error_message: None,
        },
        _ => fail(TaxIdError::InvalidCheckDigit, Some(normalized)),
    }
}

/// Why a register number failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegisterNumberError {
    Empty,
    InvalidFormat,
}

impl RegisterNumberError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Empty => "register number must not be empty",
            Self::InvalidFormat => {
                "expected register number like 'HRB 12345 B', 'HRA 76543' or 'PR 2451'"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegisterNumberReport {
    pub valid: bool,
    pub normalized: Option<String>,
    pub error: Option<RegisterNumberError>,
    pub error_message: Option<String>,
}

/// Validate a German commercial register number.
///
/// Accepted shapes (case-insensitive, flexible whitespace):
/// `HRB 12345 B`, `HRB12345B`, `HRA 76543`, `PR 2451`, `GnR 88`, `VR 12`.
/// The register itself assigns sequential numbers without check digits, so a
/// strict structural check is the strongest verifiable rule here.
pub fn validate_register_number(raw: &str) -> RegisterNumberReport {
    let fail = |error: RegisterNumberError, normalized: Option<String>| RegisterNumberReport {
        valid: false,
        normalized,
        error: Some(error),
        error_message: Some(error.message().to_string()),
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return fail(RegisterNumberError::Empty, None);
    }

    let upper = trimmed.to_ascii_uppercase();
    let compact: String = upper.split_whitespace().collect();

    let mut parts = compact.char_indices().peekable();
    let mut prefix = String::new();
    while let Some((_, c)) = parts.peek().copied() {
        if c.is_ascii_alphabetic() {
            prefix.push(c);
            parts.next();
        } else {
            break;
        }
    }
    // Canonical display form for each accepted prefix (compact is uppercased).
    const VALID_PREFIXES: [(&str, &str); 6] = [
        ("HRB", "HRB"),
        ("HRA", "HRA"),
        ("HR", "HR"),
        ("PR", "PR"),
        ("GNR", "GnR"),
        ("VR", "VR"),
    ];
    let canonical_prefix = match VALID_PREFIXES
        .iter()
        .find(|(code, _)| *code == prefix.as_str())
    {
        Some((_, display)) => *display,
        None => return fail(RegisterNumberError::InvalidFormat, None),
    };

    let rest = &compact[prefix.len()..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() < 2 || digits.len() > 6 {
        return fail(RegisterNumberError::InvalidFormat, None);
    }
    let suffix: String = rest[digits.len()..].chars().collect();
    if !suffix.is_empty()
        && (suffix.len() > 1 || !suffix.chars().next().unwrap().is_ascii_alphabetic())
    {
        return fail(RegisterNumberError::InvalidFormat, None);
    }

    let normalized = match suffix.is_empty() {
        true => format!("{canonical_prefix} {digits}"),
        false => format!(
            "{canonical_prefix} {digits} {}",
            suffix.chars().next().unwrap()
        ),
    };
    RegisterNumberReport {
        valid: true,
        normalized: Some(normalized),
        error: None,
        error_message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_valid_vat_id() {
        // Computed by the official product method: DE12345678 + check 8.
        let r = validate_tax_id("DE123456788");
        assert!(r.valid, "{r:?}");
        assert_eq!(r.normalized.as_deref(), Some("DE123456788"));
    }

    #[test]
    fn check_digit_zero_is_invalid_by_rule() {
        // 11111111 -> sum 30 -> check digit 0, which is never permitted.
        let r = validate_tax_id("DE111111110");
        assert!(!r.valid);
        assert_eq!(r.error, Some(TaxIdError::InvalidCheckDigit));
    }

    #[test]
    fn wrong_check_digit_rejected() {
        let r = validate_tax_id("DE123456789");
        assert!(!r.valid);
        assert_eq!(r.error, Some(TaxIdError::InvalidCheckDigit));
    }

    #[test]
    fn normalizes_separators_and_case() {
        let r = validate_tax_id("  de123.456.78 8 ");
        assert!(r.valid, "{r:?}");
        assert_eq!(r.normalized.as_deref(), Some("DE123456788"));
    }

    #[test]
    fn format_errors() {
        for bad in [
            "",
            "   ",
            "ATU12345678",
            "DE12345678",
            "DE1234567890",
            "DE12345678A",
            "XX123456789",
        ] {
            let r = validate_tax_id(bad);
            assert!(!r.valid, "expected failure for {bad:?}, got {r:?}");
            assert!(r.error.is_some());
            assert!(r.error_message.is_some());
        }
        assert_eq!(validate_tax_id("").error, Some(TaxIdError::Empty));
    }

    #[test]
    fn fuzz_survives_arbitrary_input() {
        // Robustness: never panic, always report.
        for bad in ["💥", "DE", "DE000000000", "de-/-.", "\u{0}DE123456788"] {
            let _ = validate_tax_id(bad);
        }
    }

    #[test]
    fn register_number_valid_shapes() {
        for (input, expected) in [
            ("HRB 12345 B", "HRB 12345 B"),
            ("hrb12345b", "HRB 12345 B"),
            ("HRA 76543", "HRA 76543"),
            ("PR 2451", "PR 2451"),
            ("GnR 88", "GnR 88"),
            ("VR 12", "VR 12"),
            ("HR 12", "HR 12"),
        ] {
            let r = validate_register_number(input);
            assert!(r.valid, "expected valid for {input:?}, got {r:?}");
            assert_eq!(r.normalized.as_deref(), Some(expected), "input: {input}");
        }
    }

    #[test]
    fn register_number_invalid_shapes() {
        for bad in [
            "",
            "   ",
            "XYZ 12345",
            "HRB",
            "HRB 1 B",       // number too short
            "HRB 1234567 B", // number too long
            "HRB 12345 BB",  // suffix too long
            "HRB 12.345",
        ] {
            let r = validate_register_number(bad);
            assert!(!r.valid, "expected invalid for {bad:?}, got {r:?}");
            assert!(r.error_message.is_some());
        }
        assert_eq!(
            validate_register_number("").error,
            Some(RegisterNumberError::Empty)
        );
    }
}
