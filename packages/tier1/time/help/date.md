# date

## Summary
Parse text into a date

## In brief
- `date("2026-10-16")`: text written as ISO 8601 writes a day, year-month-day.
- `date("16/10/2026", format: "%d/%m/%Y")`: text written another way, with the format that says how.
- `date(2026, 10, 16)`: the day that a year, a month and a day name, given as numbers.
- `date(year, month)`: the first day of that month.
- `sales.add(d = date(sold))`: any of them for every row of a column.

## Description
Reads `s` as a calendar day. Without `format:` the text must be ISO 8601, `2026-10-16`, the one
form that is never ambiguous; with `format:` it must match the format, written as `strftime`
writes it: `%Y` a four-digit year, `%y` a two-digit year (69 to 99 as 1969 to 1999, 00 to 68 as
2000 to 2068), `%m` the month as one or two digits, `%b` or `%B` the English month name in any
case, full or as three letters, `%d` the day as one or two digits, and `%%` a percent sign; white
space in the format matches any white space. A format is never guessed, as section 4.1 of the
Language Reference says of `import`. Text that does not read as a date, or an impossible date
such as `2026-02-30`, becomes `NA` with a warning (W0203); `NA` gives `NA`. Given a column, `date`
applies row by row and counts the rows that became `NA` in one warning.

Given numbers, `date(year, month, day)` is the day they name, and `date(year, month)` the first
of the month: the way from a file that carries `year` and `month`, or `year` and `quarter`, as
numbers to a `Date` that plots on a date axis and declares a panel's time. The three are Ints or
Int columns, row by row. Numbers that name no day, a month 13 or a 30 February, become `NA` with
a warning (W0203), as text does; `NA` in any of them gives `NA`.

## Arguments
- s: The text, or a column of text; or, with a month after it, the year as an Int.
- month: With a year in `s`, the month, 1 to 12.
- day: With a year and a month, the day of the month; 1 unless given.
- format: The format the text is written in, when it is not ISO 8601. Not given with numbers.

## Value
A `Date`; a column of dates when `s` is a column.

## Missing values
`NA` gives `NA`, in the text or in any of the numbers, and nothing is printed for it. Text that cannot be read and numbers that name no day become `NA` as well, and those are counted in a warning (W0203), so that a value missing in the data is told from one that the reading lost. No row is dropped.

## Details
Exact: the calendar is the proleptic Gregorian calendar of section 3.8 of the Language Reference,
years 1 to 9999, computed in integer arithmetic, so every target gives the same day.

## A common mistake
Giving text written day first, as much of the world writes a date, with no format. `16/10/2026` is not ISO 8601, and `05/02/2026` could be 5 February or 2 May, so nothing is guessed; the value becomes `NA` and the warning says so:

> date("16/10/2026")
NA
warning[W0203]: text that is not a date in ISO 8601 became NA (line 14)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`

Say how the text is written: `date("16/10/2026", format: "%d/%m/%Y")` is 2026-10-16.

## Coming from R or Stata
| date(s) | as.Date(s) | date(s, "YMD") |
| date(s, format: "%d/%m/%Y") | as.Date(s, format = "%d/%m/%Y") | date(s, "DMY") |
| date(y, m, d) | as.Date(ISOdate(y, m, d)) | mdy(m, d, y) |
| date(y, m) | as.Date(ISOdate(y, m, 1)) | mdy(m, 1, y) |

R's `as.Date` with no format tries year-month-day with hyphens and then with slashes, and reads
what fits: `as.Date("16/10/2026")` is 20 October of the year 16, with no warning. Under `%Y` it
takes a two-digit year as it stands, so `26-10-16` is the year 26. Here both become `NA` with a
warning. Stata's `date()` takes a mask of letters in place of a format and gives a number of
days since 1 January 1960, which reads as a date only under a display format (`format %td`);
here a `Date` is a type of its own and prints as a date. `mdy` takes the month first.

## Related tasks
- For a date with a time of day, use `datetime`.
- For the year, the month or the day of a date as numbers, use `year`, `month` or `day`.
- For the first day of a date's month, quarter or year, use `start_of`.
- For a date some days or months later, use `add_time`.
- To write a date as text in a pattern of your own, use `format`.

## See also
datetime, to, year, month, day

## Examples
// ISO 8601 needs no format.
> date("2026-10-16")
2026-10-16

// Day first, with the format that says so.
> date("16/10/2026", format: "%d/%m/%Y")
2026-10-16

// The English name of a month, in full or as three letters.
> date("Oct 16, 2026", format: "%b %d, %Y")
2026-10-16

// A two-digit year under `%y`: 69 to 99 are 1969 to 1999, the rest 2000 to 2068.
> date("16.10.26", format: "%d.%m.%y")
2026-10-16

// A day that does not exist becomes `NA`, with a warning.
> date("2026-02-30")
NA
warning[W0203]: text that is not a date in ISO 8601 became NA (line 5)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`

// Row by row on a column: one warning counts the rows that could not be read.
> let sales = frame(sold = ["2026-01-05", "5/2/2026", "2026-03-01"])
> sales.add(d = date(sold))
frame · 3 rows × 2 variables

      sold         d
  1   2026-01-05   2026-01-05
  2   5/2/2026     NA
  3   2026-03-01   2026-03-01
warning[W0203]: text that is not a date in ISO 8601 became NA in 1 row (line 7)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`

// From numbers: a year, a month and a day.
> date(2026, 3, 15)
2026-03-15

// A year and a month give the first of the month.
> date(2026, 3)
2026-03-01

// Numbers that name no day become `NA`, with a warning.
> date(2026, 2, 30)
NA
warning[W0203]: a year, a month and a day that name no date became NA (line 10)
  = help: a month runs from 1 to 12 and a day to the length of its month; the calendar runs from year 1 to 9999

// The shipped wages carry the year as a number; as a date it can be the time of a panel or the axis of a plot.
> dataset(wages).head(3).select([worker, year, wage]).add(period = date(year, 1))
frame · 3 rows × 4 variables · panel worker × year

      worker   year    wage   period
  1        1   2015   18.08   2015-01-01
  2        1   2016   17.28   2016-01-01
  3        1   2017   20.93   2017-01-01

// A year and a quarter, as numbers, to the first day of the quarter.
> let quarters = frame(year = [2025, 2025, 2026], qtr = [3, 4, 1], sales = [10, 12, 9])
> quarters.add(period = date(year, qtr * 3 - 2))
frame · 3 rows × 4 variables

      year   qtr   sales   period
  1   2025     3      10   2025-07-01
  2   2025     4      12   2025-10-01
  3   2026     1       9   2026-01-01
