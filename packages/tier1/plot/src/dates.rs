//! The scale of dates (Language Reference, section 7.6, "Scales"): the limits of an axis that
//! holds a Date or a DateTime variable, its breaks and the labels of the breaks.
//!
//! A place on the scale is a count of microseconds since 1970-01-01 00:00:00 UTC, the unit of
//! a DateTime; a Date stands at the midnight that starts it. Equal times are equal distances
//! on the page, so the scale places its values as a linear one does, and what differs is
//! where the breaks fall: on the calendar. A break is the start of a year, of a half-year, a
//! quarter or a month, a Monday, a day, or a round time of the clock, whichever gives about
//! five intervals between the limits. The labels are written as the ISO 8601 forms tables
//! use (section 3.8), cut to the part the breaks differ in: `2024` where every break starts a
//! year, `2024-03` where every break starts a month, `2024-03-05` for days, and the time of
//! the clock, with the day before it when the breaks fall on more than one.
//!
//! Everything here is whole-number arithmetic on the calendar of the kernel, but for the
//! choice of the step, which multiplies two doubles; it gives the same breaks on every target.

use zeff_dates::{MAX_DAYS, MIN_DAYS, civil_from_days, days_from_civil, weekday_of};

use crate::scale::{INTERVALS, MARGIN};

/// The microseconds of a second, a minute, an hour and a day.
pub const SECOND: i64 = 1_000_000;
pub const MINUTE: i64 = 60 * SECOND;
pub const HOUR: i64 = 60 * MINUTE;
pub const DAY: i64 = 24 * HOUR;

/// The mean length of a year of the Gregorian calendar, 365.2425 days, and a twelfth of it:
/// what a step of years or months is taken to last when the step is chosen.
const YEAR: i64 = 31_556_952 * SECOND;
const MONTH: i64 = YEAR / 12;

/// The first and the last instants of the calendar, years 1 to 9999.
const FIRST: i64 = MIN_DAYS as i64 * DAY;
const LAST: i64 = MAX_DAYS as i64 * DAY + (DAY - 1);

/// The distance between two neighbouring breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// A number of microseconds that divides a day: the breaks are counted from midnight.
    Clock(i64),
    /// Every day, or with 7 every Monday.
    Days(i64),
    /// The first of every month, or of every third or sixth, counted from January.
    Months(i64),
    /// The first of January of every year, or of the years that are multiples of the step.
    Years(i64),
}

impl Step {
    /// About how long the step lasts, in microseconds.
    fn length(self) -> f64 {
        (match self {
            Step::Clock(micros) => micros,
            Step::Days(days) => days * DAY,
            Step::Months(months) => months * MONTH,
            Step::Years(years) => years * YEAR,
        }) as f64
    }
}

/// The steps a scale of dates chooses among, from the shortest to the longest.
fn ladder() -> Vec<Step> {
    let mut steps: Vec<Step> = Vec::new();
    // Under a second: 1, 2 and 5 times a power of ten, as on a linear scale.
    let mut unit: i64 = 1;
    while unit < SECOND {
        steps.extend([1, 2, 5].map(|times| Step::Clock(times * unit)));
        unit *= 10;
    }
    steps.extend([1, 5, 15, 30].map(|times| Step::Clock(times * SECOND)));
    steps.extend([1, 5, 15, 30].map(|times| Step::Clock(times * MINUTE)));
    steps.extend([1, 3, 6, 12].map(|times| Step::Clock(times * HOUR)));
    steps.extend([Step::Days(1), Step::Days(7)]);
    steps.extend([Step::Months(1), Step::Months(3), Step::Months(6)]);
    let mut unit: i64 = 1;
    while unit <= 1000 {
        steps.extend([1, 2, 5].map(|times| Step::Years(times * unit)));
        unit *= 10;
    }
    steps
}

/// The limits of an axis of dates whose values run from `extent.0` to `extent.1`: the extent
/// with 5% of its width added on each side, as on a linear axis. An extent of one value is
/// widened by a day on each side where the axis holds whole days, and by an hour where it
/// holds times; with no value the limits are the first day of 1970.
pub fn limits(extent: Option<(f64, f64)>, whole_days: bool) -> (f64, f64) {
    let Some((lo, hi)) = extent else {
        return (0.0, DAY as f64);
    };
    if !(lo.is_finite() && hi.is_finite() && lo <= hi) {
        return (0.0, DAY as f64);
    }
    if lo == hi {
        let half = if whole_days { DAY } else { HOUR } as f64;
        return (lo - half, hi + half);
    }
    let pad = MARGIN * (hi - lo);
    (lo - pad, hi + pad)
}

/// The places of a step between two instants, in increasing order; nothing where there would
/// be more than two hundred.
fn places(step: Step, lo: i64, hi: i64) -> Vec<i64> {
    const MOST: usize = 200;
    let mut found: Vec<i64> = Vec::new();
    if lo > hi {
        return found;
    }
    // The day an instant falls on.
    let day_of = |micros: i64| micros.div_euclid(DAY) as i32;
    match step {
        Step::Clock(every) => {
            let mut at = lo.div_euclid(every) * every;
            if at < lo {
                at += every;
            }
            if (hi - at) / every >= MOST as i64 {
                return found;
            }
            while at <= hi {
                found.push(at);
                at += every;
            }
        }
        Step::Days(every) => {
            let mut day = i64::from(day_of(lo));
            if day * DAY < lo {
                day += 1;
            }
            if every == 7 {
                // On to the next Monday.
                day += i64::from((8 - weekday_of(day as i32)) % 7);
            }
            if (hi / DAY - day) / every >= MOST as i64 {
                return found;
            }
            while day * DAY <= hi {
                found.push(day * DAY);
                day += every;
            }
        }
        Step::Months(every) => {
            let (year, month, _) = civil_from_days(day_of(lo));
            let mut index = (i64::from(year) * 12 + i64::from(month) - 1).div_euclid(every) * every;
            while found.len() < MOST {
                let (year, month) = (index.div_euclid(12) as i32, index.rem_euclid(12) as u32 + 1);
                index += every;
                let Some(day) = days_from_civil(year, month, 1) else {
                    if year > 9999 {
                        break;
                    }
                    continue;
                };
                let at = i64::from(day) * DAY;
                if at > hi {
                    break;
                }
                if at >= lo {
                    found.push(at);
                }
            }
        }
        Step::Years(every) => {
            let (year, _, _) = civil_from_days(day_of(lo));
            let mut year = i64::from(year).div_euclid(every) * every;
            while found.len() < MOST {
                let this = year as i32;
                year += every;
                let Some(day) = days_from_civil(this, 1, 1) else {
                    if this > 9999 {
                        break;
                    }
                    continue;
                };
                let at = i64::from(day) * DAY;
                if at > hi {
                    break;
                }
                if at >= lo {
                    found.push(at);
                }
            }
        }
    }
    found
}

/// The breaks of an axis of dates between its limits, in increasing order.
///
/// The step is the one of the ladder nearest, in proportion, to a fifth of the width. Where
/// fewer than two of its places fall between the limits, the next shorter step is tried. An
/// axis of whole days has no step shorter than a day, unless its limits hold no midnight at
/// all. Where no step has a place between the limits, the breaks are the limits themselves.
pub fn breaks(lo: f64, hi: f64, whole_days: bool) -> Vec<f64> {
    let raw = hi / INTERVALS - lo / INTERVALS;
    if !(raw.is_finite() && raw > 0.0) {
        return vec![lo, hi];
    }
    // The instants of the calendar the limits hold: the first at or after the lower, the
    // last at or before the upper. A cast of a double to a whole number stops at its ends.
    let first = (lo.ceil() as i64).max(FIRST);
    let last = (hi.floor() as i64).min(LAST);
    let steps = ladder();
    let floor = if whole_days {
        steps
            .iter()
            .position(|step| *step == Step::Days(1))
            .unwrap_or(0)
    } else {
        0
    };
    // The first step that `raw` is nearer to than to the next one: below the geometric mean
    // of the two.
    let chosen = (floor..steps.len())
        .find(|&i| {
            steps
                .get(i + 1)
                .is_none_or(|next| raw * raw < steps[i].length() * next.length())
        })
        .unwrap_or(steps.len() - 1);
    let as_doubles = |found: Vec<i64>| found.into_iter().map(|at| at as f64).collect();
    let mut one: Option<Vec<i64>> = None;
    for i in (0..=chosen).rev() {
        let found = places(steps[i], first, last);
        if found.len() >= 2 {
            return as_doubles(found);
        }
        if found.len() == 1 && one.is_none() {
            one = Some(found);
        }
        // An axis of whole days goes below a day only for want of any break at all.
        if i == floor && one.is_some() {
            break;
        }
    }
    match one {
        Some(found) => as_doubles(found),
        None => vec![lo, hi],
    }
}

/// An instant of the scale as the day it falls on and the microseconds since its midnight,
/// held within the calendar.
fn parts(at: f64) -> (i32, i64) {
    let micros = if at.is_finite() {
        (at.round() as i64).clamp(FIRST, LAST)
    } else {
        0
    };
    (micros.div_euclid(DAY) as i32, micros.rem_euclid(DAY))
}

/// The time of the clock, `14:05`, with the seconds where `seconds` and their fraction to
/// `digits` places where there are any.
fn clock(within: i64, seconds: bool, digits: usize) -> String {
    let whole = within / SECOND;
    let mut out = format!("{:02}:{:02}", whole / 3600, (whole / 60) % 60);
    if seconds {
        out.push_str(&format!(":{:02}", whole % 60));
    }
    if digits > 0 {
        let fraction = (within % SECOND) / 10_i64.pow(6 - digits as u32);
        out.push_str(&format!(".{fraction:0digits$}"));
    }
    out
}

/// The labels of the breaks of an axis of dates: the ISO 8601 form, cut to the part the
/// breaks differ in. The year where every break starts a year; the year and the month where
/// every break starts a month; the day where every break is a midnight; otherwise the time of
/// the clock, to the minute, the second or the fraction the breaks need, with the day before
/// it where the breaks fall on more than one day.
pub fn labels(breaks: &[f64]) -> Vec<String> {
    let at: Vec<(i32, i64)> = breaks.iter().map(|b| parts(*b)).collect();
    let civil: Vec<(i32, u32, u32)> = at.iter().map(|(day, _)| civil_from_days(*day)).collect();
    if at.iter().all(|(_, within)| *within == 0) {
        let yearly = civil.iter().all(|(_, month, day)| *month == 1 && *day == 1);
        let monthly = civil.iter().all(|(_, _, day)| *day == 1);
        return civil
            .iter()
            .map(|(year, month, day)| {
                if yearly {
                    format!("{year:04}")
                } else if monthly {
                    format!("{year:04}-{month:02}")
                } else {
                    format!("{year:04}-{month:02}-{day:02}")
                }
            })
            .collect();
    }
    let seconds = at.iter().any(|(_, within)| within % MINUTE != 0);
    let digits = if at.iter().all(|(_, within)| within % SECOND == 0) {
        0
    } else if at.iter().all(|(_, within)| within % 1000 == 0) {
        3
    } else {
        6
    };
    let days = at.windows(2).any(|pair| pair[0].0 != pair[1].0);
    at.iter()
        .zip(&civil)
        .map(|((_, within), (year, month, day))| {
            let time = clock(*within, seconds || digits > 0, digits);
            if days {
                format!("{year:04}-{month:02}-{day:02} {time}")
            } else {
                time
            }
        })
        .collect()
}

/// An instant as a reader is told it, in the summary of a plot and in a message: the day,
/// and with `timed` the time of the clock to the second after it.
pub fn shown(at: f64, timed: bool) -> String {
    let (day, within) = parts(at);
    let (year, month, date) = civil_from_days(day);
    let mut out = format!("{year:04}-{month:02}-{date:02}");
    if timed {
        out.push(' ');
        out.push_str(&clock(within, true, 0));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(year: i32, month: u32, date: u32) -> f64 {
        (i64::from(days_from_civil(year, month, date).unwrap()) * DAY) as f64
    }

    fn at(year: i32, month: u32, date: u32, hour: i64, minute: i64) -> f64 {
        day(year, month, date) + (hour * HOUR + minute * MINUTE) as f64
    }

    fn drawn(lo: f64, hi: f64, whole_days: bool) -> Vec<String> {
        labels(&breaks(lo, hi, whole_days))
    }

    #[test]
    fn the_ladder_rises() {
        let steps = ladder();
        assert!(
            steps
                .windows(2)
                .all(|pair| pair[0].length() < pair[1].length())
        );
        // Every step of the clock divides a day, so that its breaks count from midnight.
        for step in &steps {
            if let Step::Clock(every) = step {
                assert_eq!(DAY % every, 0, "{step:?}");
            }
        }
    }

    #[test]
    fn years_months_and_days_as_the_range_asks() {
        assert_eq!(
            drawn(day(2015, 1, 1), day(2020, 1, 1), true),
            ["2015", "2016", "2017", "2018", "2019", "2020"]
        );
        assert_eq!(
            drawn(day(1990, 6, 1), day(2021, 3, 1), true),
            ["1995", "2000", "2005", "2010", "2015", "2020"]
        );
        assert_eq!(
            drawn(day(2023, 11, 20), day(2025, 2, 10), true),
            ["2024-01", "2024-04", "2024-07", "2024-10", "2025-01"]
        );
        assert_eq!(
            drawn(day(2024, 1, 10), day(2024, 6, 20), true),
            ["2024-02", "2024-03", "2024-04", "2024-05", "2024-06"]
        );
        assert_eq!(
            drawn(day(2024, 3, 1), day(2024, 3, 6), true),
            [
                "2024-03-01",
                "2024-03-02",
                "2024-03-03",
                "2024-03-04",
                "2024-03-05",
                "2024-03-06"
            ]
        );
    }

    #[test]
    fn weeks_start_on_a_monday() {
        // 5 February 2024 was a Monday.
        let found = breaks(day(2024, 2, 1), day(2024, 3, 20), true);
        assert_eq!(found[0], day(2024, 2, 5));
        assert!(
            found
                .windows(2)
                .all(|pair| pair[1] - pair[0] == 7.0 * DAY as f64)
        );
        assert!(found.iter().all(|b| weekday_of(parts(*b).0) == 1));
        assert_eq!(labels(&found)[0], "2024-02-05");
    }

    #[test]
    fn a_range_across_a_leap_day_holds_it() {
        let found = drawn(day(2024, 2, 26), day(2024, 3, 3), true);
        assert!(found.contains(&"2024-02-29".to_string()), "{found:?}");
        let common = drawn(day(2023, 2, 26), day(2023, 3, 3), true);
        assert!(!common.iter().any(|label| label.ends_with("02-29")));
        assert_eq!(found.len(), common.len() + 1);
    }

    #[test]
    fn one_day_stands_between_its_neighbours() {
        let one = day(2024, 3, 5);
        let (lo, hi) = limits(Some((one, one)), true);
        assert_eq!((lo, hi), (day(2024, 3, 4), day(2024, 3, 6)));
        assert_eq!(
            drawn(lo, hi, true),
            ["2024-03-04", "2024-03-05", "2024-03-06"]
        );
        // One instant has an hour on each side.
        let instant = at(2024, 3, 5, 14, 20);
        let (lo, hi) = limits(Some((instant, instant)), false);
        assert_eq!(drawn(lo, hi, false), ["13:30", "14:00", "14:30", "15:00"]);
    }

    #[test]
    fn a_century_and_more_is_marked_in_round_years() {
        assert_eq!(
            drawn(day(1900, 1, 1), day(2000, 1, 1), true),
            ["1900", "1920", "1940", "1960", "1980", "2000"]
        );
        assert_eq!(
            drawn(day(1, 1, 1), day(9999, 12, 31), true),
            ["2000", "4000", "6000", "8000"]
        );
    }

    #[test]
    fn the_clock_is_marked_at_round_times() {
        assert_eq!(
            drawn(at(2024, 3, 5, 9, 0), at(2024, 3, 5, 13, 0), false),
            ["09:00", "10:00", "11:00", "12:00", "13:00"]
        );
        assert_eq!(
            drawn(at(2024, 3, 5, 8, 0), at(2024, 3, 5, 20, 0), false),
            ["09:00", "12:00", "15:00", "18:00"]
        );
        // More than one day: the day stands before the time.
        assert_eq!(
            drawn(at(2024, 3, 5, 9, 0), at(2024, 3, 6, 17, 0), false),
            [
                "2024-03-05 12:00",
                "2024-03-05 18:00",
                "2024-03-06 00:00",
                "2024-03-06 06:00",
                "2024-03-06 12:00"
            ]
        );
        assert_eq!(
            drawn(at(2024, 3, 5, 9, 0), at(2024, 3, 5, 9, 2), false),
            ["09:00:00", "09:00:30", "09:01:00", "09:01:30", "09:02:00"]
        );
        let start = at(2024, 3, 5, 9, 0);
        assert_eq!(
            drawn(start, start + 4000.0, false),
            [
                "09:00:00.000",
                "09:00:00.001",
                "09:00:00.002",
                "09:00:00.003",
                "09:00:00.004"
            ]
        );
        assert_eq!(
            drawn(start, start + 2.0, false),
            ["09:00:00.000000", "09:00:00.000001", "09:00:00.000002"]
        );
    }

    #[test]
    fn whole_days_are_never_marked_by_the_hour() {
        // A day and a half of whole days: the two midnights, not the hours between.
        assert_eq!(
            drawn(day(2024, 3, 5) - 1.0, at(2024, 3, 6, 12, 0), true),
            ["2024-03-05", "2024-03-06"]
        );
        // With no midnight between the limits, the clock is all there is.
        let found = drawn(at(2024, 3, 5, 6, 0), at(2024, 3, 5, 18, 0), true);
        assert_eq!(found, ["06:00", "12:00", "18:00"]);
    }

    #[test]
    fn limits_that_hold_nothing_are_their_own_breaks() {
        assert_eq!(breaks(5.0, 5.0, false), [5.0, 5.0]);
        assert_eq!(breaks(7.0, 3.0, false), [7.0, 3.0]);
        assert_eq!(breaks(0.25, 0.75, false), [0.25, 0.75]);
        // Beyond the calendar there is no day to mark.
        let beyond = LAST as f64 * 4.0;
        assert_eq!(breaks(beyond, beyond * 2.0, true), [beyond, beyond * 2.0]);
        assert_eq!(labels(&[f64::NAN]), ["1970"]);
    }

    #[test]
    fn the_limits_add_five_per_cent_on_each_side() {
        let (lo, hi) = limits(Some((day(2024, 1, 1), day(2024, 1, 21))), true);
        assert_eq!((lo, hi), (day(2023, 12, 31), day(2024, 1, 22)));
        assert_eq!(limits(None, true), (0.0, DAY as f64));
        assert_eq!(limits(Some((f64::NAN, 1.0)), true), (0.0, DAY as f64));
    }

    #[test]
    fn an_instant_is_shown_as_its_day_or_to_the_second() {
        assert_eq!(shown(at(2024, 3, 5, 14, 5), false), "2024-03-05");
        assert_eq!(shown(at(2024, 3, 5, 14, 5), true), "2024-03-05 14:05:00");
        assert_eq!(shown(-1.0, true), "1969-12-31 23:59:59");
    }
}
