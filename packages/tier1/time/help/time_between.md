# time_between

## Summary
Whole units of time from start to end

## In brief
- `time_between(hired, left)`: the days from one date to another.
- `time_between(hired, left, unit: year)`: whole years of service; `month` and `week` likewise.
- `time_between(arrived, served, unit: minute)`: whole minutes between two times; `second` and
  `hour` likewise.

## Description
The number of whole units from `start` to `end`, as an Int: positive when `end` is later,
negative when it is earlier, and zero when less than one unit lies between them. A part of a unit
is dropped, never rounded, so 90 seconds is 1 minute, 23 hours is 0 days and 13 days is 1 week,
in either direction.

Two dates are counted by their days. Two times are counted by their instants, so the time of day
matters: from 9:00 on one day to 8:00 on the next is 0 days and 23 hours. The units of the clock,
`second`, `minute` and `hour`, need a time on both sides. Given columns, `time_between` applies
row by row.

## Arguments
- start: A `Date` or a `DateTime`, or a column of either.
- end: A value of the same type as `start`, or a column of it.
- unit: What to count: `second`, `minute`, `hour`, `day`, `week`, `month` or `year`; `day`
  unless given. The unit is never chosen from the data.

## Value
An Int; a column of Ints when either argument is a column.

## Missing values
`NA` in `start` or in `end` gives `NA` for that row, without a note: a time that is not known
has no distance to another. Nothing is dropped.

## Details
### Months and years
A month has no fixed length, so months are counted on the calendar: the count is the largest
number of months that can be added to `start` without passing `end`. Adding a month keeps the
day of the month and, where the month is too short, takes its last day, as `add_time` does;
so 31 January to 28 February is 1 month, and 31 January to 27 February is 0. A year is twelve
such months, so 29 February 2024 to 28 February 2025 is 1 year. Counting backwards follows the
same rule from `start`: 28 February back to 31 January is 0 months, since one month before
28 February is 28 January.

### A week
A week is seven days, counted from `start`, not a calendar week: Friday to the next Monday is
0 weeks. For the week of the year a date falls in, use `week`.

### Exactness
Integer arithmetic on the day counts of two dates, or on the microseconds of two times; no
result is rounded.

## A common mistake
Counting hours between two dates. A `Date` has no time of day, and the engine does not put
midnight in its place without being asked:

> time_between(date("2026-01-31"), date("2026-02-02"), unit: hour)
error[Z0401]: `unit: hour` counts between two times, but `start` and `end` are dates
  = help: a Date has no time of day: count with `unit: day`, or give two DateTimes (`to(d, DateTime)` is midnight of the day)

Count days, or read the columns as times with `datetime`.

## Coming from R or Stata
| time_between(a, b) | as.integer(b - a) | b - a |
| time_between(a, b, unit: hour) | trunc(difftime(b, a, units = "hours")) | clockdiff(a, b, "hour") |
| time_between(a, b, unit: month) | lubridate::interval(a, b) %/% months(1) | datediff(a, b, "month") |
| time_between(a, b, unit: year) | lubridate::interval(a, b) %/% years(1) | datediff(a, b, "year") |

R's `difftime` picks its unit from the data unless `units =` is given, and keeps the fraction;
here the unit is `day` unless written and the result is whole. The month and year counts agree
with `lubridate`'s, month-end clamping included. Stata's `mofd(b) - mofd(a)` counts calendar
months crossed instead, 31 January to 1 February being one.

## Related tasks
- To move a date by days, weeks, months or years, use `add_time`.
- For the age of something on the day the script runs, give the run's date as the end, with `today`.
- For the year, month or weekday a date falls in, use `year`, `month` or `weekday`.
- To turn text into a date or a time first, use `date` or `datetime`.

## See also
add_time, date, datetime, today

## Examples
// Days between two dates.
> time_between(date("2026-01-31"), date("2026-03-01"))
29

// Negative when the end is the earlier one.
> time_between(date("2026-03-01"), date("2026-01-31"))
-29

// Whole months follow the calendar: 31 January to the last day of February is one month.
> time_between(date("2026-01-31"), date("2026-02-28"), unit: month)
1

// One day short of it is none.
> time_between(date("2026-01-31"), date("2026-02-27"), unit: month)
0

// A year is twelve such months, so six years from a 29 February end on 28 February.
> time_between(date("2020-02-29"), date("2026-02-28"), unit: year)
6

// Between two times the instants are counted, here in whole hours.
> let opened = datetime("2026-03-02T09:15:00")
> let closed = datetime("2026-03-03T08:45:30")
> time_between(opened, closed, unit: hour)
23

// The same two times are less than a day apart.
> time_between(opened, closed)
0

// And in seconds.
> time_between(opened, closed, unit: second)
84630

// Row by row on columns: the waiting time of each visit in whole minutes, NA where a time is missing.
> let visits = frame(arrived = [datetime("2026-03-02T09:15"), datetime("2026-03-02T10:02"), NA], served = [datetime("2026-03-02T09:41"), datetime("2026-03-02T11:30"), datetime("2026-03-02T12:00")])
> visits.add(wait = time_between(arrived, served, unit: minute))
frame · 3 rows × 3 variables

      arrived               served                wait
  1   2026-03-02T09:15:00   2026-03-02T09:41:00     26
  2   2026-03-02T10:02:00   2026-03-02T11:30:00     88
  3   NA                    2026-03-02T12:00:00     NA

// Whole weeks of a loan, counted in sevens of days from the day it was issued.
> let loans = frame(issued = [date("2026-01-05"), date("2026-02-20")], repaid = [date("2026-03-01"), date("2026-02-20")])
> loans.add(weeks = time_between(issued, repaid, unit: week))
frame · 2 rows × 3 variables

      issued       repaid       weeks
  1   2026-01-05   2026-03-01       7
  2   2026-02-20   2026-02-20       0
