//! Engine kernel: the calendar, and the parsing and printing of dates and times (Language
//! Reference, section 3.8, "Time"; section 4.1, "Import").
//!
//! * The calendar is the proleptic Gregorian calendar, years 1 to 9999, with no leap seconds.
//! * A `Date` is a count of days since 1970-01-01 (Arrow's Date32); a `DateTime` is a count of
//!   microseconds since 1970-01-01 00:00:00 UTC (Arrow's Timestamp in microseconds).
//! * Text is read only when its form is unambiguous (ISO 8601) or when a format names the form;
//!   a format is never guessed.
//!
//! Everything here is integer arithmetic, so it gives the same answer on every target.

use std::fmt::Write;

const MICROS_PER_SECOND: i64 = 1_000_000;
const MICROS_PER_DAY: i64 = 86_400 * MICROS_PER_SECOND;

/// Whether `year` is a leap year in the Gregorian calendar.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// The number of days in a month; `None` for a month outside 1 to 12.
pub fn days_in_month(year: i32, month: u32) -> Option<u32> {
    Some(match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return None,
    })
}

/// Whether `year-month-day` is a day of the calendar (years 1 to 9999).
pub fn is_valid_date(year: i32, month: u32, day: u32) -> bool {
    (1..=9999).contains(&year) && day >= 1 && days_in_month(year, month).is_some_and(|n| day <= n)
}

/// The days since 1970-01-01 of a calendar day, or `None` if it is not a day of the calendar.
pub fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i32> {
    if !is_valid_date(year, month, day) {
        return None;
    }
    // Howard Hinnant's algorithm, in integers.
    let y = i64::from(year) - i64::from(month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let mp = u64::from(if month > 2 { month - 3 } else { month + 9 });
    let doy = (153 * mp + 2) / 5 + u64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i32::try_from(era * 146_097 + doe as i64 - 719_468).ok()
}

/// The calendar day of a count of days since 1970-01-01, as (year, month, day). Defined for every
/// count that [`days_from_civil`] can produce; outside years 1 to 9999 the result follows the
/// same arithmetic but is not a day Zeff accepts.
pub fn civil_from_days(days: i32) -> (i32, u32, u32) {
    let z = i64::from(days) + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if month <= 2 { y + 1 } else { y };
    (year as i32, month, day)
}

/// The smallest and largest day counts of the calendar Zeff accepts: 0001-01-01 and 9999-12-31.
pub const MIN_DAYS: i32 = -719_162;
pub const MAX_DAYS: i32 = 2_932_896;

/// A wall-clock time to the microsecond.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Time {
    hour: u32,
    minute: u32,
    second: u32,
    micro: u32,
}

impl Time {
    fn micros_of_day(self) -> i64 {
        (i64::from(self.hour) * 3600 + i64::from(self.minute) * 60 + i64::from(self.second))
            * MICROS_PER_SECOND
            + i64::from(self.micro)
    }
}

fn is_valid_time(hour: u32, minute: u32, second: u32) -> bool {
    hour < 24 && minute < 60 && second < 60 // no leap seconds
}

/// The microseconds since 1970-01-01 00:00:00 UTC of a day and a time.
fn micros_from(days: i32, time: Time) -> i64 {
    i64::from(days) * MICROS_PER_DAY + time.micros_of_day()
}

/// The day and the microseconds within it of a timestamp.
fn split_micros(micros: i64) -> (i32, i64) {
    let days = micros.div_euclid(MICROS_PER_DAY);
    (days as i32, micros.rem_euclid(MICROS_PER_DAY))
}

// ----- reading digits -----

/// The ASCII digits at the start of `text`, at most `max`; the value and how many were read.
fn digits(text: &str, max: usize) -> Option<(u32, usize)> {
    let bytes = text.as_bytes();
    let mut value: u32 = 0;
    let mut n = 0;
    while n < max && n < bytes.len() && bytes[n].is_ascii_digit() {
        value = value
            .checked_mul(10)?
            .checked_add(u32::from(bytes[n] - b'0'))?;
        n += 1;
    }
    (n > 0).then_some((value, n))
}

/// Exactly `n` ASCII digits.
fn fixed_digits(text: &str, n: usize) -> Option<u32> {
    let (value, read) = digits(text, n)?;
    (read == n).then_some(value)
}

// ----- ISO 8601 -----

/// `YYYY-MM-DD`, exactly, as days since 1970-01-01.
pub fn parse_iso_date(text: &str) -> Option<i32> {
    let (days, rest) = iso_date_prefix(text)?;
    rest.is_empty().then_some(days)
}

fn iso_date_prefix(text: &str) -> Option<(i32, &str)> {
    let year = fixed_digits(text, 4)?;
    let rest = text.get(4..)?.strip_prefix('-')?;
    let month = fixed_digits(rest, 2)?;
    let rest = rest.get(2..)?.strip_prefix('-')?;
    let day = fixed_digits(rest, 2)?;
    let rest = rest.get(2..)?;
    let days = days_from_civil(i32::try_from(year).ok()?, month, day)?;
    Some((days, rest))
}

/// A parsed ISO 8601 date and time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedDateTime {
    /// Microseconds since 1970-01-01 00:00:00 UTC.
    pub micros: i64,
    /// Whether the text carried `Z` or an offset, which was applied to reach UTC (W0702).
    pub had_offset: bool,
}

/// `HH:MM`, `HH:MM:SS` or `HH:MM:SS.ffffff` (one to nine fraction digits, kept to the
/// microsecond); the time and the rest of the text.
fn time_prefix(text: &str) -> Option<(Time, &str)> {
    let hour = fixed_digits(text, 2)?;
    let rest = text.get(2..)?.strip_prefix(':')?;
    let minute = fixed_digits(rest, 2)?;
    let mut rest = rest.get(2..)?;
    let mut second = 0;
    let mut micro = 0;
    if let Some(after) = rest.strip_prefix(':') {
        second = fixed_digits(after, 2)?;
        rest = after.get(2..)?;
        if let Some(fraction) = rest.strip_prefix('.') {
            let (value, read) = digits(fraction, 9)?;
            // Scale to six places: pad short fractions, truncate long ones.
            micro = if read <= 6 {
                value * 10u32.pow((6 - read) as u32)
            } else {
                value / 10u32.pow((read - 6) as u32)
            };
            rest = fraction.get(read..)?;
        }
    }
    is_valid_time(hour, minute, second).then_some((
        Time {
            hour,
            minute,
            second,
            micro,
        },
        rest,
    ))
}

/// `Z`, `+HH:MM`, `-HH:MM`, `+HHMM` or `+HH`: the offset in minutes east of UTC.
fn offset(text: &str) -> Option<i64> {
    if text == "Z" || text == "z" {
        return Some(0);
    }
    let sign = match text.chars().next()? {
        '+' => 1,
        '-' => -1,
        _ => return None,
    };
    let rest = &text[1..];
    let hours = fixed_digits(rest, 2)?;
    let rest = &rest[2..];
    let minutes = if rest.is_empty() {
        0
    } else {
        let rest = rest.strip_prefix(':').unwrap_or(rest);
        let m = fixed_digits(rest, 2)?;
        if rest.len() != 2 {
            return None;
        }
        m
    };
    (hours < 24 && minutes < 60).then_some(sign * (i64::from(hours) * 60 + i64::from(minutes)))
}

/// ISO 8601 date and time: `YYYY-MM-DDTHH:MM[:SS[.f]]`, with `T` or a space between date and
/// time and an optional `Z` or offset, which is applied to give UTC.
pub fn parse_iso_datetime(text: &str) -> Option<ParsedDateTime> {
    let (days, rest) = iso_date_prefix(text)?;
    let rest = rest
        .strip_prefix('T')
        .or_else(|| rest.strip_prefix(' '))
        .or_else(|| rest.strip_prefix('t'))?;
    let (time, rest) = time_prefix(rest)?;
    let mut micros = micros_from(days, time);
    let had_offset = !rest.is_empty();
    if had_offset {
        let minutes_east = offset(rest)?;
        micros -= minutes_east * 60 * MICROS_PER_SECOND;
    }
    Some(ParsedDateTime { micros, had_offset })
}

// ----- printing -----

/// `YYYY-MM-DD`.
pub fn format_iso_date(days: i32) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `YYYY-MM-DDTHH:MM:SS`, with `.ffffff` when the microseconds are not zero.
pub fn format_iso_datetime(micros: i64) -> String {
    let (days, within) = split_micros(micros);
    let seconds = within / MICROS_PER_SECOND;
    let micro = within % MICROS_PER_SECOND;
    let mut out = format_iso_date(days);
    let _ = write!(
        out,
        "T{:02}:{:02}:{:02}",
        seconds / 3600,
        (seconds / 60) % 60,
        seconds % 60
    );
    if micro != 0 {
        let _ = write!(out, ".{micro:06}");
    }
    out
}

// ----- formats -----

/// One piece of a format.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    /// `%Y`: a four-digit year.
    Year,
    /// `%y`: a two-digit year, 69–99 as 1969–1999 and 00–68 as 2000–2068.
    ShortYear,
    /// `%m`: the month as one or two digits.
    Month,
    /// `%b` or `%B`: the month by its English name or three-letter abbreviation, in any case.
    MonthName,
    /// `%d`: the day as one or two digits.
    Day,
    /// `%H`: the hour as one or two digits.
    Hour,
    /// `%M`: the minute as one or two digits.
    Minute,
    /// `%S`: the second as one or two digits, with an optional fraction `.f`.
    Second,
    /// A run of white space in the format matches a run of white space in the text.
    Space,
    /// Text that must appear as written.
    Literal(String),
}

/// Why a format cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    Empty,
    /// `%` followed by a letter the formats do not have.
    UnknownDirective(String),
    /// A `%` at the very end.
    TrailingPercent,
    /// A directive given twice, or both `%Y` and `%y`.
    Repeated(String),
    /// No year, month or day.
    NoDate,
    /// A minute or second without an hour.
    TimeWithoutHour,
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Empty => write!(f, "the format is empty"),
            FormatError::UnknownDirective(d) => {
                write!(f, "`{d}` is not a directive of a date format")
            }
            FormatError::TrailingPercent => write!(f, "the format ends with a `%`"),
            FormatError::Repeated(d) => write!(f, "`{d}` appears twice"),
            FormatError::NoDate => write!(f, "the format has no year, month or day"),
            FormatError::TimeWithoutHour => {
                write!(f, "the format has a minute or second but no hour")
            }
        }
    }
}

impl std::error::Error for FormatError {}

/// A date or date-and-time format, written as `strftime` writes it: `"%d/%m/%Y"`,
/// `"%Y-%m-%d %H:%M"`, `"%b %d, %Y"`. The directives are `%Y`, `%y`, `%m`, `%b`, `%B`, `%d`,
/// `%H`, `%M`, `%S` and `%%`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateFormat {
    pattern: String,
    pieces: Vec<Piece>,
    has_time: bool,
}

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

impl DateFormat {
    pub fn new(pattern: &str) -> Result<DateFormat, FormatError> {
        if pattern.is_empty() {
            return Err(FormatError::Empty);
        }
        let mut pieces: Vec<Piece> = Vec::new();
        let mut chars = pattern.chars().peekable();
        let mut literal = String::new();
        let mut seen: Vec<&str> = Vec::new();
        let flush = |literal: &mut String, pieces: &mut Vec<Piece>| {
            if !literal.is_empty() {
                pieces.push(Piece::Literal(std::mem::take(literal)));
            }
        };
        while let Some(c) = chars.next() {
            if c == '%' {
                let Some(d) = chars.next() else {
                    return Err(FormatError::TrailingPercent);
                };
                if d == '%' {
                    literal.push('%');
                    continue;
                }
                let (piece, key) = match d {
                    'Y' => (Piece::Year, "year"),
                    'y' => (Piece::ShortYear, "year"),
                    'm' => (Piece::Month, "month"),
                    'b' | 'B' => (Piece::MonthName, "month"),
                    'd' => (Piece::Day, "day"),
                    'H' => (Piece::Hour, "hour"),
                    'M' => (Piece::Minute, "minute"),
                    'S' => (Piece::Second, "second"),
                    other => return Err(FormatError::UnknownDirective(format!("%{other}"))),
                };
                if seen.contains(&key) {
                    return Err(FormatError::Repeated(format!("%{d}")));
                }
                seen.push(key);
                flush(&mut literal, &mut pieces);
                pieces.push(piece);
            } else if c.is_whitespace() {
                flush(&mut literal, &mut pieces);
                if pieces.last() != Some(&Piece::Space) {
                    pieces.push(Piece::Space);
                }
            } else {
                literal.push(c);
            }
        }
        flush(&mut literal, &mut pieces);
        if !(seen.contains(&"year") && seen.contains(&"month") && seen.contains(&"day")) {
            return Err(FormatError::NoDate);
        }
        let has_time = seen.contains(&"hour");
        if !has_time && (seen.contains(&"minute") || seen.contains(&"second")) {
            return Err(FormatError::TimeWithoutHour);
        }
        Ok(DateFormat {
            pattern: pattern.to_string(),
            pieces,
            has_time,
        })
    }

    /// The format as written.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Whether the format has an hour, so that it reads a `DateTime` and not a `Date`.
    pub fn has_time(&self) -> bool {
        self.has_time
    }

    /// Read `text` in full against the format: the day, and the time (midnight when the format
    /// has none).
    fn read(&self, text: &str) -> Option<(i32, Time)> {
        let mut rest = text;
        let (mut year, mut month, mut day) = (None, None, None);
        let (mut hour, mut minute, mut second, mut micro) = (0, 0, 0, 0);
        for piece in &self.pieces {
            match piece {
                Piece::Year => {
                    year = Some(i32::try_from(fixed_digits(rest, 4)?).ok()?);
                    rest = &rest[4..];
                }
                Piece::ShortYear => {
                    let short = fixed_digits(rest, 2)?;
                    year = Some(if short >= 69 {
                        1900 + short as i32
                    } else {
                        2000 + short as i32
                    });
                    rest = &rest[2..];
                }
                Piece::Month => {
                    let (value, n) = digits(rest, 2)?;
                    month = Some(value);
                    rest = &rest[n..];
                }
                Piece::MonthName => {
                    let lower: String = rest
                        .chars()
                        .take_while(|c| c.is_ascii_alphabetic())
                        .collect();
                    let folded = lower.to_ascii_lowercase();
                    let index = MONTHS.iter().position(|m| {
                        *m == folded || (folded.len() == 3 && m.starts_with(folded.as_str()))
                    })?;
                    month = Some(index as u32 + 1);
                    rest = &rest[lower.len()..];
                }
                Piece::Day => {
                    let (value, n) = digits(rest, 2)?;
                    day = Some(value);
                    rest = &rest[n..];
                }
                Piece::Hour => {
                    let (value, n) = digits(rest, 2)?;
                    hour = value;
                    rest = &rest[n..];
                }
                Piece::Minute => {
                    let (value, n) = digits(rest, 2)?;
                    minute = value;
                    rest = &rest[n..];
                }
                Piece::Second => {
                    let (value, n) = digits(rest, 2)?;
                    second = value;
                    rest = &rest[n..];
                    if let Some(fraction) = rest.strip_prefix('.') {
                        let (value, read) = digits(fraction, 9)?;
                        micro = if read <= 6 {
                            value * 10u32.pow((6 - read) as u32)
                        } else {
                            value / 10u32.pow((read - 6) as u32)
                        };
                        rest = &fraction[read..];
                    }
                }
                Piece::Space => {
                    let trimmed = rest.trim_start();
                    if trimmed.len() == rest.len() {
                        return None;
                    }
                    rest = trimmed;
                }
                Piece::Literal(text) => {
                    rest = rest.strip_prefix(text.as_str())?;
                }
            }
        }
        if !rest.is_empty() || !is_valid_time(hour, minute, second) {
            return None;
        }
        let days = days_from_civil(year?, month?, day?)?;
        Some((
            days,
            Time {
                hour,
                minute,
                second,
                micro,
            },
        ))
    }

    /// `text` as days since 1970-01-01; the time part, if the format has one, is dropped.
    pub fn parse_date(&self, text: &str) -> Option<i32> {
        self.read(text).map(|(days, _)| days)
    }

    /// `text` as microseconds since 1970-01-01 00:00:00 UTC, taking the text as UTC.
    pub fn parse_datetime(&self, text: &str) -> Option<i64> {
        self.read(text).map(|(days, time)| micros_from(days, time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_days() {
        assert_eq!(days_from_civil(1970, 1, 1), Some(0));
        assert_eq!(days_from_civil(1969, 12, 31), Some(-1));
        assert_eq!(days_from_civil(2000, 3, 1), Some(11_017));
        assert_eq!(days_from_civil(2026, 10, 16), Some(20_742));
        assert_eq!(days_from_civil(1, 1, 1), Some(MIN_DAYS));
        assert_eq!(days_from_civil(9999, 12, 31), Some(MAX_DAYS));
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(civil_from_days(MIN_DAYS), (1, 1, 1));
        assert_eq!(civil_from_days(MAX_DAYS), (9999, 12, 31));
    }

    #[test]
    fn leap_years_and_month_lengths() {
        assert!(
            is_leap_year(2000) && is_leap_year(2024) && !is_leap_year(1900) && !is_leap_year(2023)
        );
        assert_eq!(days_in_month(2024, 2), Some(29));
        assert_eq!(days_in_month(2023, 2), Some(28));
        assert_eq!(days_in_month(2023, 13), None);
        assert_eq!(days_from_civil(2026, 2, 30), None, "an impossible date");
        assert_eq!(days_from_civil(2023, 2, 29), None);
        assert_eq!(days_from_civil(2024, 2, 29), Some(19_782));
        assert_eq!(
            days_from_civil(0, 1, 1),
            None,
            "year 0 is outside the calendar"
        );
        assert_eq!(days_from_civil(10_000, 1, 1), None);
        assert_eq!(days_from_civil(2026, 0, 1), None);
        assert_eq!(days_from_civil(2026, 1, 0), None);
    }

    #[test]
    fn every_day_of_the_calendar_round_trips() {
        for days in (MIN_DAYS..=MAX_DAYS).step_by(97) {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), Some(days), "{y}-{m}-{d}");
        }
        for days in [MIN_DAYS, MIN_DAYS + 1, -1, 0, 1, MAX_DAYS - 1, MAX_DAYS] {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), Some(days));
        }
    }

    #[test]
    fn iso_dates_are_read_strictly() {
        assert_eq!(parse_iso_date("2026-10-16"), Some(20_742));
        assert_eq!(parse_iso_date("0001-01-01"), Some(MIN_DAYS));
        for bad in [
            "2026-1-16",
            "26-10-16",
            "2026/10/16",
            "2026-10-16T",
            "2026-10-16 ",
            " 2026-10-16",
            "2026-02-30",
            "20261016",
            "2026-10",
            "",
        ] {
            assert_eq!(parse_iso_date(bad), None, "{bad}");
        }
    }

    #[test]
    fn iso_datetimes_with_and_without_offsets() {
        let day = i64::from(20_742) * MICROS_PER_DAY;
        let at = |h: i64, m: i64, s: i64| day + (h * 3600 + m * 60 + s) * MICROS_PER_SECOND;
        assert_eq!(
            parse_iso_datetime("2026-10-16T13:45:30"),
            Some(ParsedDateTime {
                micros: at(13, 45, 30),
                had_offset: false
            })
        );
        assert_eq!(
            parse_iso_datetime("2026-10-16 13:45:30"),
            Some(ParsedDateTime {
                micros: at(13, 45, 30),
                had_offset: false
            })
        );
        assert_eq!(
            parse_iso_datetime("2026-10-16T13:45"),
            Some(ParsedDateTime {
                micros: at(13, 45, 0),
                had_offset: false
            })
        );
        assert_eq!(
            parse_iso_datetime("2026-10-16T13:45:30.5"),
            Some(ParsedDateTime {
                micros: at(13, 45, 30) + 500_000,
                had_offset: false
            })
        );
        assert_eq!(
            parse_iso_datetime("2026-10-16T13:45:30.123456789"),
            Some(ParsedDateTime {
                micros: at(13, 45, 30) + 123_456,
                had_offset: false
            })
        );
        assert_eq!(
            parse_iso_datetime("2026-10-16T13:45:30Z"),
            Some(ParsedDateTime {
                micros: at(13, 45, 30),
                had_offset: true
            })
        );
        assert_eq!(
            parse_iso_datetime("2026-10-16T13:45:30+02:00"),
            Some(ParsedDateTime {
                micros: at(11, 45, 30),
                had_offset: true
            })
        );
        assert_eq!(
            parse_iso_datetime("2026-10-16T13:45:30-0530"),
            Some(ParsedDateTime {
                micros: at(19, 15, 30),
                had_offset: true
            })
        );
        assert_eq!(
            parse_iso_datetime("2026-10-16T23:45:30-01"),
            Some(ParsedDateTime {
                micros: at(24, 45, 30),
                had_offset: true
            }),
            "the next day"
        );
        for bad in [
            "2026-10-16T24:00:00",
            "2026-10-16T13:60:00",
            "2026-10-16T13:45:60",
            "2026-10-16T13",
            "2026-10-16T13:45:30+25:00",
            "2026-10-16T13:45:30 Z",
            "2026-10-16T13:45:30X",
        ] {
            assert_eq!(parse_iso_datetime(bad), None, "{bad}");
        }
    }

    #[test]
    fn printing_is_iso() {
        assert_eq!(format_iso_date(0), "1970-01-01");
        assert_eq!(format_iso_date(-1), "1969-12-31");
        assert_eq!(format_iso_date(MIN_DAYS), "0001-01-01");
        assert_eq!(format_iso_datetime(0), "1970-01-01T00:00:00");
        assert_eq!(format_iso_datetime(-1), "1969-12-31T23:59:59.999999");
        let t = parse_iso_datetime("2026-10-16T13:45:30.25").unwrap().micros;
        assert_eq!(format_iso_datetime(t), "2026-10-16T13:45:30.250000");
        let t = parse_iso_datetime("2026-10-16T13:45:30+02:00")
            .unwrap()
            .micros;
        assert_eq!(format_iso_datetime(t), "2026-10-16T11:45:30");
    }

    #[test]
    fn formats() {
        let f = DateFormat::new("%d/%m/%Y").unwrap();
        assert!(!f.has_time());
        assert_eq!(f.parse_date("16/10/2026"), Some(20_742));
        assert_eq!(f.parse_date("6/1/2026"), days_from_civil(2026, 1, 6));
        assert_eq!(f.parse_date("16-10-2026"), None);
        assert_eq!(f.parse_date("16/10/2026 "), None);
        assert_eq!(f.parse_date("30/02/2026"), None);
        let us = DateFormat::new("%m/%d/%Y").unwrap();
        assert_eq!(us.parse_date("10/16/2026"), Some(20_742));
        assert_eq!(us.parse_date("16/10/2026"), None);
        let short = DateFormat::new("%d.%m.%y").unwrap();
        assert_eq!(short.parse_date("16.10.26"), Some(20_742));
        assert_eq!(short.parse_date("01.01.69"), days_from_civil(1969, 1, 1));
        assert_eq!(short.parse_date("01.01.68"), days_from_civil(2068, 1, 1));
        let words = DateFormat::new("%b %d, %Y").unwrap();
        assert_eq!(words.parse_date("Oct 16, 2026"), Some(20_742));
        assert_eq!(words.parse_date("OCTOBER 16, 2026"), Some(20_742));
        assert_eq!(words.parse_date("Octo 16, 2026"), None);
        let spaced = DateFormat::new("%Y %m %d").unwrap();
        assert_eq!(spaced.parse_date("2026   10 16"), Some(20_742));
        assert_eq!(spaced.parse_date("20261016"), None);
        let percent = DateFormat::new("%Y%%%m%%%d").unwrap();
        assert_eq!(percent.parse_date("2026%10%16"), Some(20_742));
    }

    #[test]
    fn formats_with_a_time() {
        let f = DateFormat::new("%Y-%m-%d %H:%M").unwrap();
        assert!(f.has_time());
        let expected = parse_iso_datetime("2026-10-16T13:45").unwrap().micros;
        assert_eq!(f.parse_datetime("2026-10-16 13:45"), Some(expected));
        assert_eq!(
            f.parse_date("2026-10-16 13:45"),
            Some(20_742),
            "the date part alone"
        );
        let s = DateFormat::new("%d/%m/%Y %H:%M:%S").unwrap();
        assert_eq!(
            s.parse_datetime("16/10/2026 13:45:30.5"),
            Some(expected + 30 * MICROS_PER_SECOND + 500_000)
        );
        assert_eq!(s.parse_datetime("16/10/2026 25:00:00"), None);
    }

    #[test]
    fn bad_formats_are_refused_with_the_reason() {
        assert_eq!(DateFormat::new(""), Err(FormatError::Empty));
        assert_eq!(
            DateFormat::new("%d/%m/%Q"),
            Err(FormatError::UnknownDirective("%Q".into()))
        );
        assert_eq!(
            DateFormat::new("%d/%m/%Y%"),
            Err(FormatError::TrailingPercent)
        );
        assert_eq!(
            DateFormat::new("%d/%m/%Y %y"),
            Err(FormatError::Repeated("%y".into()))
        );
        assert_eq!(
            DateFormat::new("%d/%d/%Y"),
            Err(FormatError::Repeated("%d".into()))
        );
        assert_eq!(DateFormat::new("%d/%m"), Err(FormatError::NoDate));
        assert_eq!(DateFormat::new("%H:%M"), Err(FormatError::NoDate));
        assert_eq!(
            DateFormat::new("%d/%m/%Y %M"),
            Err(FormatError::TimeWithoutHour)
        );
        assert_eq!(DateFormat::new("%d/%m/%Y").unwrap().pattern(), "%d/%m/%Y");
    }
}
