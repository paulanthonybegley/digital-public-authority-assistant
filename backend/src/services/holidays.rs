//! German statutory public holidays (§ 6 Feiertagsgesetze der Länder).
//!
//! Scope: national holidays plus the well-known state-specific holidays.
//! Moving holidays (Easter-based, Buß- und Bettag) are computed, never hardcoded.

use chrono::{Datelike, Duration, NaiveDate, Weekday};

/// The 16 German federal states (Bundesländer) as ISO 3166-2 codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FederalState {
    BB,
    BE,
    BW,
    BY,
    HB,
    HE,
    MV,
    NI,
    NW,
    RP,
    SL,
    SN,
    ST,
    SH,
    TH,
}

impl FederalState {
    /// Parse a federal state from a case-insensitive ISO code (`"BY"`, `"by"`).
    pub fn parse(raw: &str) -> Result<Self, String> {
        let code = raw.trim().to_ascii_uppercase();
        Ok(match code.as_str() {
            "BB" => Self::BB,
            "BE" => Self::BE,
            "BW" => Self::BW,
            "BY" => Self::BY,
            "HB" => Self::HB,
            "HE" => Self::HE,
            "MV" => Self::MV,
            "NI" => Self::NI,
            "NW" => Self::NW,
            "RP" => Self::RP,
            "SL" => Self::SL,
            "SN" => Self::SN,
            "ST" => Self::ST,
            "SH" => Self::SH,
            "TH" => Self::TH,
            _ => {
                return Err(format!(
                    "unknown federal state '{raw}' (expected one of: BB, BE, BW, BY, HB, HE, MV, NI, NW, RP, SL, SN, ST, SH, TH)"
                ))
            }
        })
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::BB => "BB",
            Self::BE => "BE",
            Self::BW => "BW",
            Self::BY => "BY",
            Self::HB => "HB",
            Self::HE => "HE",
            Self::MV => "MV",
            Self::NI => "NI",
            Self::NW => "NW",
            Self::RP => "RP",
            Self::SL => "SL",
            Self::SN => "SN",
            Self::ST => "ST",
            Self::SH => "SH",
            Self::TH => "TH",
        }
    }
}

/// Easter Sunday (Gregorian calendar), anonymous computus.
fn easter_sunday(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = ((h + l - 7 * m + 114) / 31) as u32;
    let day = (((h + l - 7 * m + 114) % 31) + 1) as u32;
    NaiveDate::from_ymd_opt(year, month, day).expect("computus yields a valid date")
}

/// Buß- und Bettag: the Wednesday strictly before 23 November (SN only).
fn repentance_day(year: i32) -> NaiveDate {
    let mut d = NaiveDate::from_ymd_opt(year, 11, 22).expect("valid date");
    while d.weekday() != Weekday::Wed {
        d -= Duration::days(1);
    }
    d
}

/// All public holidays of `year` for `state`, sorted by date.
pub fn public_holidays(year: i32, state: FederalState) -> Vec<(NaiveDate, &'static str)> {
    use FederalState as S;

    let mut holidays: Vec<(NaiveDate, &'static str)> = vec![
        (NaiveDate::from_ymd_opt(year, 1, 1).unwrap(), "Neujahr"),
        (
            NaiveDate::from_ymd_opt(year, 5, 1).unwrap(),
            "Tag der Arbeit",
        ),
        (
            NaiveDate::from_ymd_opt(year, 10, 3).unwrap(),
            "Tag der Deutschen Einheit",
        ),
        (
            NaiveDate::from_ymd_opt(year, 12, 25).unwrap(),
            "1. Weihnachtstag",
        ),
        (
            NaiveDate::from_ymd_opt(year, 12, 26).unwrap(),
            "2. Weihnachtstag",
        ),
    ];

    // Easter-based holidays observed in (almost) every state.
    let easter = easter_sunday(year);
    holidays.push((easter - Duration::days(2), "Karfreitag"));
    holidays.push((easter + Duration::days(1), "Ostermontag"));
    holidays.push((easter + Duration::days(39), "Christi Himmelfahrt"));
    holidays.push((easter + Duration::days(50), "Pfingstmontag"));

    // State-specific holidays.
    if matches!(state, S::BY | S::BW | S::ST) {
        holidays.push((
            NaiveDate::from_ymd_opt(year, 1, 6).unwrap(),
            "Heilige Drei Könige",
        ));
    }
    if state == S::BE {
        holidays.push((
            NaiveDate::from_ymd_opt(year, 3, 8).unwrap(),
            "Internationaler Frauentag",
        ));
    }
    if matches!(state, S::BY | S::BW | S::HE | S::NW | S::RP | S::SL) {
        holidays.push((easter + Duration::days(60), "Fronleichnam"));
    }
    // Reformation Day (nationwide since 2017, exceptions: SH, BE, HB, NI).
    if !matches!(state, S::SH | S::BE | S::HB | S::NI) {
        holidays.push((
            NaiveDate::from_ymd_opt(year, 10, 31).unwrap(),
            "Reformationstag",
        ));
    }
    if matches!(state, S::BY | S::BW | S::NW | S::RP | S::SL) {
        holidays.push((
            NaiveDate::from_ymd_opt(year, 11, 1).unwrap(),
            "Allerheiligen",
        ));
    }
    if state == S::SN {
        holidays.push((repentance_day(year), "Buß- und Bettag"));
    }

    holidays.sort_by_key(|(date, _)| *date);
    holidays
}

/// Whether `date` is a public holiday in `state`.
pub fn is_public_holiday(date: NaiveDate, state: FederalState) -> bool {
    public_holidays(date.year(), state)
        .iter()
        .any(|(h, _)| *h == date)
}

/// Is `date` a business day (neither weekend nor public holiday)?
pub fn is_business_day(date: NaiveDate, state: FederalState) -> bool {
    !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) && !is_public_holiday(date, state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easter_known_years() {
        assert_eq!(
            easter_sunday(2024),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()
        );
        assert_eq!(
            easter_sunday(2025),
            NaiveDate::from_ymd_opt(2025, 4, 20).unwrap()
        );
        assert_eq!(
            easter_sunday(2026),
            NaiveDate::from_ymd_opt(2026, 4, 5).unwrap()
        );
        assert_eq!(
            easter_sunday(2000),
            NaiveDate::from_ymd_opt(2000, 4, 23).unwrap()
        );
    }

    #[test]
    fn fixed_holidays_present_in_every_state() {
        let new_year = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let labour = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let unity = NaiveDate::from_ymd_opt(2026, 10, 3).unwrap();
        for state in [
            FederalState::BY,
            FederalState::BE,
            FederalState::NI,
            FederalState::SH,
        ] {
            assert!(is_public_holiday(new_year, state));
            assert!(is_public_holiday(labour, state));
            assert!(is_public_holiday(unity, state));
        }
    }

    #[test]
    fn state_specific_epiphany_only_in_by_bw_st() {
        let epiphany = NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(); // Tuesday
        assert!(is_public_holiday(epiphany, FederalState::BY));
        assert!(is_public_holiday(epiphany, FederalState::BW));
        assert!(is_public_holiday(epiphany, FederalState::ST));
        assert!(!is_public_holiday(epiphany, FederalState::BE));
        assert!(!is_public_holiday(epiphany, FederalState::NI));
    }

    #[test]
    fn reformation_day_exceptions() {
        let reformation = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap(); // Saturday
        assert!(is_public_holiday(reformation, FederalState::SN));
        assert!(is_public_holiday(reformation, FederalState::BB));
        assert!(!is_public_holiday(reformation, FederalState::NI));
        assert!(!is_public_holiday(reformation, FederalState::SH));
        assert!(!is_public_holiday(reformation, FederalState::BE));
    }

    #[test]
    fn repentance_day_is_wednesday_before_nov_23() {
        // 2025-11-19 is the Wednesday strictly before 2025-11-23 (Sunday).
        assert_eq!(
            repentance_day(2025),
            NaiveDate::from_ymd_opt(2025, 11, 19).unwrap()
        );
        // 2024-11-20: 23 Nov 2024 is a Saturday.
        assert_eq!(
            repentance_day(2024),
            NaiveDate::from_ymd_opt(2024, 11, 20).unwrap()
        );
        assert_eq!(repentance_day(2023).weekday(), Weekday::Wed);
    }

    #[test]
    fn state_codes_roundtrip() {
        assert_eq!(FederalState::parse("by").unwrap(), FederalState::BY);
        assert_eq!(FederalState::parse(" NW ").unwrap(), FederalState::NW);
        assert_eq!(FederalState::parse("BY").unwrap().code(), "BY");
        assert!(FederalState::parse("XX").is_err());
        assert!(FederalState::parse("").is_err());
    }

    #[test]
    fn business_day_skips_weekend_and_holidays() {
        // 2026-05-01 (Labour Day) is a Friday; Saturday/Sunday follow.
        assert!(!is_business_day(
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            FederalState::BE
        ));
        assert!(!is_business_day(
            NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            FederalState::BE
        ));
        assert!(is_business_day(
            NaiveDate::from_ymd_opt(2026, 5, 4).unwrap(),
            FederalState::BE
        ));
    }
}
