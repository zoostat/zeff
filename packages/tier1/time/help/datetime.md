# datetime

## Summary
Parse text into a date and time

## In brief
- `datetime("2026-10-16T09:30")`: text written as ISO 8601 writes a date and time.
- `datetime("16/10/2026 09:30", format: "%d/%m/%Y %H:%M")`: text written another way, with its format.
- `datetime(1763200800, epoch: seconds)`: a number of seconds since the start of 1970, as logs carry it.
- `datetime(x, epoch: milliseconds)`: the same counted in milliseconds.
- `visits.add(at = datetime(stamp))`: any of them for every row of a column.

## Description
Reads `s` as a date and time to the microsecond. Without `format:` the text must be ISO 8601:
`2026-10-16T09:30`, `2026-10-16 09:30:00` or `2026-10-16T09:30:00.25`, with `T` or a space
between the date and the time, seconds and a fraction optional, and a date alone read as midnight;
a trailing `Z` or an offset such as `+02:00` is applied to reach UTC and counted in a warning
(W0702), since a `DateTime` carries no zone in 1.0 (section 3.8 of the Language Reference). With
`format:` the text must match the format, written as `strftime` writes it, with `%H`, `%M` and
`%S` for the time beside the date directives of `date`; a format with no hour reads midnight.
Text that does not read, or an impossible date or time, becomes `NA` with a warning (W0203);
`NA` gives `NA`. Given a column, `datetime` applies row by row and counts the rows in one
warning. Every value is read as UTC and never shifted afterwards.

With `epoch:` the first argument is a number of Unix time instead of text: `seconds` or
`milliseconds` since 1970-01-01 00:00:00 UTC, negative before it, as logs and web data carry
it. A whole number is exact; a number with a fraction is read to the nearest microsecond. A
number outside years 1 to 9999 becomes `NA` with a warning (W0203). Text is refused with
`epoch:`, and a number without it, so that a number such as 20261016 is never taken for a count
of seconds.

## Arguments
- s: The text, or a column of text; with `epoch:`, a number or a column of numbers.
- format: The format the text is written in, when it is not ISO 8601.
- epoch: What a number in `s` counts since 1970-01-01 00:00:00 UTC: `seconds` or `milliseconds`.
  Not given with `format:`.

## Value
A `DateTime`; a column of them when `s` is a column.

## Missing values
`NA` gives `NA`, in the text or in the number, and nothing is printed for it. Text that cannot be read and a number outside the years 1 to 9999 become `NA` as well, and those are counted in a warning (W0203), so that a value missing in the data is told from one that the reading lost. No row is dropped.

## Details
Exact: seconds are kept to the microsecond and the calendar is that of `date`, so every target
gives the same instant. A `DateTime` prints in ISO 8601 with `T`, `2026-10-16T09:30:00`, and
with its fraction when it has one.

## A common mistake
Giving a number with nothing to say what it counts: a stamp from a log, or a date written as the digits 20261016. It could be seconds, milliseconds or the digits of a day, so it is refused:

> datetime(1763200800)
error[Z0401]: `datetime` reads text, but `s` is an Int
  = help: a number is Unix time only when the call says so: `epoch: seconds` or `epoch: milliseconds`

Say what the number counts: `datetime(1763200800, epoch: seconds)`. A day written as digits is text with a format: `date("20261016", format: "%Y%m%d")`.

## Coming from R or Stata
| datetime(s) | as.POSIXct(s, tz = "UTC") | clock(s, "YMDhms") |
| datetime(s, format: "%d/%m/%Y %H:%M") | as.POSIXct(s, format = "%d/%m/%Y %H:%M", tz = "UTC") | clock(s, "DMYhm") |
| datetime(x, epoch: seconds) | as.POSIXct(x, origin = "1970-01-01", tz = "UTC") | x * 1000 + tc(01jan1970 00:00:00) |
| datetime(x, epoch: milliseconds) | as.POSIXct(x / 1000, origin = "1970-01-01", tz = "UTC") | x + tc(01jan1970 00:00:00) |

R's `as.POSIXct` reads text in the time zone of the session unless `tz =` is given, so one
script can give two instants on two machines; here text is read as it stands, and an offset
written in the text is applied to reach UTC, with a warning. Stata keeps a date and time as
milliseconds since 1 January 1960 and the variable must be stored as a `double`, or the seconds
are lost to rounding; here a `DateTime` is a type of its own, exact to the microsecond.

## Related tasks
- For a day with no time, use `date`.
- For the hour, the minute or the second of a time, use `hour`, `minute` or `second`.
- For the midnight that starts a time's day, or the start of its hour, use `start_of`.
- For whole hours or minutes between two times, use `time_between`.
- To make a time the day it falls on, use `to`.

## See also
date, to, year, day

## Examples
// ISO 8601 needs no format; seconds may be left out.
> datetime("2026-10-16T09:30")
2026-10-16T09:30:00

// A space in place of the `T`, and a fraction of a second.
> datetime("2026-10-16 09:30:00.25")
2026-10-16T09:30:00.250000

// A date alone is its midnight.
> datetime("2026-10-16")
2026-10-16T00:00:00

// Day first, with the format that says so.
> datetime("16/10/2026 09:30", format: "%d/%m/%Y %H:%M")
2026-10-16T09:30:00

// An offset in the text is applied to reach UTC, and the warning says so.
> datetime("2026-10-16T09:30:00+02:00")
2026-10-16T07:30:00
warning[W0702]: a time-zone offset was applied to reach UTC (line 5)
  = help: a DateTime has no zone in 1.0; the instant is kept, written in UTC

// An hour that does not exist becomes `NA`, with a warning.
> datetime("2026-10-16T25:00")
NA
warning[W0203]: text that is not a date and time in ISO 8601 became NA (line 6)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`

// Unix time in seconds.
> datetime(1763200800, epoch: seconds)
2025-11-15T10:00:00

// Unix time in milliseconds, as web data carry it.
> datetime(1763200800250, epoch: milliseconds)
2025-11-15T10:00:00.250000

// A fraction, and a time before 1970.
> datetime(-1.5, epoch: seconds)
1969-12-31T23:59:58.500000

// Row by row on a column: one warning counts the rows that could not be read.
> let visits = frame(stamp = ["2026-03-02 09:15", "2026-03-02T17:40:10", "yesterday"])
> visits.add(at = datetime(stamp))
frame · 3 rows × 2 variables

      stamp                 at
  1   2026-03-02 09:15      2026-03-02T09:15:00
  2   2026-03-02T17:40:10   2026-03-02T17:40:10
  3   yesterday             NA
warning[W0203]: text that is not a date and time in ISO 8601 became NA in 1 row (line 11)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`

// A column of Unix time.
> let hits = frame(ts = [1763200800, 1763204400, NA])
> hits.add(at = datetime(ts, epoch: seconds))
frame · 3 rows × 2 variables

              ts   at
  1   1763200800   2025-11-15T10:00:00
  2   1763204400   2025-11-15T11:00:00
  3           NA   NA
