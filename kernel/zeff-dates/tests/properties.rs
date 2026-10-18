//! Property tests of the calendar: every day round-trips, days are consecutive, and the
//! ISO forms read back what they print.

use proptest::prelude::*;
use zeff_dates::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(3000)))]

    #[test]
    fn a_day_count_round_trips_through_the_calendar(days in MIN_DAYS..=MAX_DAYS) {
        let (y, m, d) = civil_from_days(days);
        prop_assert!(is_valid_date(y, m, d));
        prop_assert_eq!(days_from_civil(y, m, d), Some(days));
        prop_assert_eq!(parse_iso_date(&format_iso_date(days)), Some(days));
    }

    #[test]
    fn the_next_day_is_one_more(days in MIN_DAYS..MAX_DAYS) {
        let (y, m, d) = civil_from_days(days);
        let (y2, m2, d2) = civil_from_days(days + 1);
        // Either the day advances within the month, or a new month (or year) begins on the 1st.
        let same_month = y == y2 && m == m2 && d2 == d + 1;
        let new_month = d2 == 1 && (m2 == m + 1 && y2 == y || (m2 == 1 && m == 12 && y2 == y + 1));
        prop_assert!(same_month || new_month, "{y}-{m}-{d} then {y2}-{m2}-{d2}");
        prop_assert!(new_month == (Some(d) == days_in_month(y, m)));
    }

    #[test]
    fn a_calendar_day_is_valid_exactly_when_it_exists(year in -5i32..10_010, month in 0u32..14, day in 0u32..33) {
        let valid = (1..=9999).contains(&year) && days_in_month(year, month).is_some_and(|n| (1..=n).contains(&day));
        prop_assert_eq!(days_from_civil(year, month, day).is_some(), valid);
    }

    #[test]
    fn a_timestamp_round_trips_through_its_text(micros in (i64::from(MIN_DAYS) * 86_400_000_000)..=(i64::from(MAX_DAYS) * 86_400_000_000 + 86_399_999_999)) {
        let text = format_iso_datetime(micros);
        let back = parse_iso_datetime(&text).unwrap();
        prop_assert_eq!(back.micros, micros);
        prop_assert!(!back.had_offset);
    }

    #[test]
    fn a_format_reads_back_what_it_describes(days in MIN_DAYS..=MAX_DAYS) {
        let (y, m, d) = civil_from_days(days);
        let f = DateFormat::new("%d/%m/%Y").unwrap();
        prop_assert_eq!(f.parse_date(&format!("{d:02}/{m:02}/{y:04}")), Some(days));
        prop_assert_eq!(f.parse_date(&format!("{d}/{m}/{y:04}")), Some(days));
        let us = DateFormat::new("%m-%d-%Y").unwrap();
        prop_assert_eq!(us.parse_date(&format!("{m:02}-{d:02}-{y:04}")), Some(days));
    }
}

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}
