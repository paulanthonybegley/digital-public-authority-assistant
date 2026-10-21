//! Statutory deadline calculation (VwVfG § 32 Abs. 2 / ZPO § 193):
//! a period ending on a Saturday, Sunday or public holiday ends on the
//! next business day.

use chrono::{Datelike, Duration, NaiveDate};

use super::holidays::{is_business_day, is_public_holiday, FederalState};

/// How the elapsed days are counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DayCounting {
    /// Calendar days: the raw end date is rolled forward to the next business day.
    #[default]
    Calendar,
    /// Business days: weekends and public holidays are skipped while counting.
    Business,
}

impl DayCounting {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Calendar => "calendar",
            Self::Business => "business",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "calendar" | "kalender" => Ok(Self::Calendar),
            "business" | "geschaeft" | "geschäft" => Ok(Self::Business),
            other => Err(format!(
                "unknown day counting '{other}' (expected 'calendar' or 'business')"
            )),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DeadlineError {
    #[error("invalid date '{0}' (expected strict YYYY-MM-DD)")]
    InvalidDate(String),
    #[error("date out of supported range 1900-01-01..=2100-12-31: {0}")]
    DateOutOfRange(String),
    #[error("days_to_add must be >= 0, got {0}")]
    NegativeDays(i64),
}

/// Why the statutory end date differs from the raw calculated date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShiftReason {
    Weekend,
    PublicHoliday,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DeadlineResult {
    /// The official (statutory) end date.
    pub date: NaiveDate,
    /// The raw date before rolling (calendar counting only).
    pub raw_date: NaiveDate,
    /// Whether the end date had to be moved.
    pub shifted: bool,
    /// Why it was moved, if it was.
    pub reason: Option<ShiftReason>,
}

/// Strict `YYYY-MM-DD` parsing: rejects chrono's lenient single-digit forms.
fn parse_date(raw: &str) -> Result<NaiveDate, DeadlineError> {
    let s = raw.trim();
    let bytes = s.as_bytes();
    let shape_ok = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..].iter().all(u8::is_ascii_digit);
    if !shape_ok {
        return Err(DeadlineError::InvalidDate(raw.to_string()));
    }
    let date = NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| DeadlineError::InvalidDate(raw.to_string()))?;
    let year = date.year();
    if !(1900..=2100).contains(&year) {
        return Err(DeadlineError::DateOutOfRange(raw.to_string()));
    }
    Ok(date)
}

/// Calculate the official deadline.
///
/// `days_to_add` calendar days are added to `start_date`; if the result falls
/// on a Saturday, Sunday or public holiday of `federal_state`, it is shifted
/// to the next business day. With [`DayCounting::Business`] only business days
/// are counted, so the result is always a business day.
pub fn calculate_deadline(
    start_date: &str,
    days_to_add: i64,
    federal_state: FederalState,
    counting: DayCounting,
) -> Result<DeadlineResult, DeadlineError> {
    if days_to_add < 0 {
        return Err(DeadlineError::NegativeDays(days_to_add));
    }
    let start = parse_date(start_date)?;

    let (raw_date, date) = match counting {
        DayCounting::Calendar => {
            let raw = start + Duration::days(days_to_add);
            let mut rolled = raw;
            loop {
                if is_business_day(rolled, federal_state) {
                    break;
                }
                rolled += Duration::days(1);
            }
            (raw, rolled)
        }
        DayCounting::Business => {
            let mut current = start;
            // A deadline period starts running the day after the triggering event.
            let mut remaining = days_to_add;
            while remaining > 0 {
                current += Duration::days(1);
                if is_business_day(current, federal_state) {
                    remaining -= 1;
                }
            }
            // Guard against pathological inputs (e.g. 100k days): still a business day.
            while !is_business_day(current, federal_state) {
                current += Duration::days(1);
            }
            (current, current)
        }
    };

    let shifted = raw_date != date;
    let reason = if shifted {
        if matches!(
            raw_date.weekday(),
            chrono::Weekday::Sat | chrono::Weekday::Sun
        ) {
            Some(ShiftReason::Weekend)
        } else if is_public_holiday(raw_date, federal_state) {
            Some(ShiftReason::PublicHoliday)
        } else {
            None
        }
    } else {
        None
    };

    Ok(DeadlineResult {
        date,
        raw_date,
        shifted,
        reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cal(start: &str, days: i64, state: FederalState) -> DeadlineResult {
        calculate_deadline(start, days, state, DayCounting::Calendar).unwrap()
    }

    #[test]
    fn plain_weekday_stays_put() {
        // 2026-01-05 is a Monday, +3 days = Thursday 2026-01-08.
        let r = cal("2026-01-05", 3, FederalState::BE);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 1, 8).unwrap());
        assert!(!r.shifted);
        assert_eq!(r.reason, None);
    }

    #[test]
    fn friday_rolls_to_monday() {
        // 2026-01-02 is a Friday; +1 = Saturday -> Monday 2026-01-05.
        let r = cal("2026-01-02", 1, FederalState::BE);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 1, 5).unwrap());
        assert!(r.shifted);
        assert_eq!(r.reason, Some(ShiftReason::Weekend));
    }

    #[test]
    fn public_holiday_rolls_forward() {
        // Labour Day 2026-05-01 is a Friday and a holiday in every state.
        let r = cal("2026-04-30", 1, FederalState::BE);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 5, 4).unwrap()); // Monday
        assert_eq!(r.reason, Some(ShiftReason::PublicHoliday));
    }

    #[test]
    fn consecutive_holiday_weekend_bridge() {
        // 2026-10-03 (Unity Day) is a Saturday; Sunday 4th; Monday 5th is a
        // business day in BE, but in BY Reformationstag chain: 2026-10-31 is a
        // Saturday (Reformationstag in BY), Sunday 11/1 -> Monday 11/2.
        let r = cal("2026-10-30", 1, FederalState::BY);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 11, 2).unwrap());
        assert_eq!(r.reason, Some(ShiftReason::Weekend));
    }

    #[test]
    fn year_end_transition() {
        // 2025-12-24 (Wed) + 7 days = 2025-12-31 (Wed), not a holiday -> stays.
        let r = cal("2025-12-24", 7, FederalState::BE);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
        assert!(!r.shifted);

        // 2025-12-26 (Fri, 2nd Christmas) + 1 day = Saturday -> Monday 29th.
        let r = cal("2025-12-26", 1, FederalState::BE);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2025, 12, 29).unwrap());

        // 2025-12-31 + 1 = New Year's Day (holiday, Thursday) -> Friday Jan 2.
        let r = cal("2025-12-31", 1, FederalState::BE);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 1, 2).unwrap());
        assert_eq!(r.reason, Some(ShiftReason::PublicHoliday));

        // 2024-12-31 + 1 = New Year's Day 2025 (Wednesday) -> Thursday Jan 2.
        let r = cal("2024-12-31", 1, FederalState::BE);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());
    }

    #[test]
    fn leap_day_start() {
        // 2028 is a leap year: 2028-02-28 + 2 = 2028-03-01 (Wednesday).
        let r = cal("2028-02-28", 2, FederalState::BE);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2028, 3, 1).unwrap());
    }

    #[test]
    fn business_day_counting_skips_weekend() {
        // Start Friday 2026-01-02, +1 business day -> Monday 2026-01-05.
        let r =
            calculate_deadline("2026-01-02", 1, FederalState::BE, DayCounting::Business).unwrap();
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 1, 5).unwrap());
        assert!(!r.shifted);
    }

    #[test]
    fn business_day_counting_skips_holiday() {
        // Start 2026-04-30 (Thu), +1 business day -> Monday 2026-05-04
        // (Friday 1st is Labour Day, weekend follows).
        let r =
            calculate_deadline("2026-04-30", 1, FederalState::BE, DayCounting::Business).unwrap();
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 5, 4).unwrap());
    }

    #[test]
    fn zero_days_returns_start_rolled() {
        let r = cal("2026-01-03", 0, FederalState::BE); // Saturday
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 1, 5).unwrap());
    }

    #[test]
    fn rejects_negative_days() {
        assert_eq!(
            calculate_deadline("2026-01-05", -1, FederalState::BE, DayCounting::Calendar),
            Err(DeadlineError::NegativeDays(-1))
        );
    }

    #[test]
    fn rejects_invalid_dates() {
        for bad in [
            "",
            "  ",
            "2026-13-01",
            "2026-02-30",
            "05.01.2026",
            "2026-1-5",
            "not-a-date",
            "20260105",
        ] {
            assert!(
                matches!(
                    calculate_deadline(bad, 1, FederalState::BE, DayCounting::Calendar),
                    Err(DeadlineError::InvalidDate(_)) | Err(DeadlineError::DateOutOfRange(_))
                ),
                "expected error for {bad:?}"
            );
        }
    }

    #[test]
    fn rejects_out_of_range_year() {
        assert!(matches!(
            calculate_deadline("1899-01-01", 1, FederalState::BE, DayCounting::Calendar),
            Err(DeadlineError::DateOutOfRange(_))
        ));
        assert!(matches!(
            calculate_deadline("2101-01-01", 1, FederalState::BE, DayCounting::Calendar),
            Err(DeadlineError::DateOutOfRange(_))
        ));
    }

    #[test]
    fn counting_mode_parses() {
        assert_eq!(
            DayCounting::parse("Calendar").unwrap(),
            DayCounting::Calendar
        );
        assert_eq!(
            DayCounting::parse("business").unwrap(),
            DayCounting::Business
        );
        assert!(DayCounting::parse("weekly").is_err());
    }
}
