# date

## Summary
Parse text into a date

## Description
Reads `s` as a calendar day. Without `format:` the text must be ISO 8601, `2026-10-16`, the one
form that is never ambiguous; with `format:` it must match the format, written as `strftime`
writes it: `%Y` a four-digit year, `%y` a two-digit year (69 to 99 as 1969 to 1999, 00 to 68 as
2000 to 2068), `%m` the month as one or two digits, `%b` or `%B` the English month name in any
case, full or as three letters, `%d` the day as one or two digits, and `%%` a percent sign; white
space in the format matches any white space. A format is never guessed, as section 4.1 of the
Language Reference says of `import`. Text that does not read as a date, or an impossible date
such as `2026-02-30`, becomes `NA` with a warning (W0203); `NA` gives `NA`. Given a column, `date`
applies row by row and counts the rows that became `NA` in one warning. R's `as.Date` reads the
same forms and also a two-digit year under `%Y`, which Zeff refuses; Stata's `date()` takes a
mask such as `"DMY"` in place of a format.

Given numbers, `date(year, month, day)` is the day they name, and `date(year, month)` the first
of the month: the way from a file that carries `year` and `month`, or `year` and `quarter`, as
numbers to a `Date` that plots on a date axis and declares a panel's time. The three are Ints or
Int columns, row by row. Numbers that name no day, a month 13 or a 30 February, become `NA` with
a warning (W0203), as text does; `NA` in any of them gives `NA`. Like R's `ISOdate(year, month,
day)` and Stata's `mdy(month, day, year)`.

## Arguments
- s: The text, or a column of text; or, with a month after it, the year as an Int.
- month: With a year in `s`, the month, 1 to 12.
- day: With a year and a month, the day of the month; 1 unless given.
- format: The format the text is written in, when it is not ISO 8601. Not given with numbers.

## Value
A `Date`; a column of dates when `s` is a column.

## Details
Exact: the calendar is the proleptic Gregorian calendar of section 3.8 of the Language Reference,
years 1 to 9999, computed in integer arithmetic, so every target gives the same day.

## See also
datetime, to, year, month, day

## Examples
> date("2026-10-16")
2026-10-16
> date("16/10/2026", format: "%d/%m/%Y")
2026-10-16
> date("Oct 16, 2026", format: "%b %d, %Y")
2026-10-16
> date("2026-02-30")
NA
warning[W0203]: text that is not a date in ISO 8601 became NA (line 4)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`
> let sales = frame(sold = ["2026-01-05", "5/2/2026", "2026-03-01"])
> sales.add(d = date(sold))
frame · 3 rows × 2 variables

      sold         d
  1   2026-01-05   2026-01-05
  2   5/2/2026     NA
  3   2026-03-01   2026-03-01
warning[W0203]: text that is not a date in ISO 8601 became NA in 1 row (line 6)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`
> date(2026, 3, 15)
2026-03-15
> date(2026, 3)
2026-03-01
> date(2026, 2, 30)
NA
warning[W0203]: a year, a month and a day that name no date became NA (line 9)
  = help: a month runs from 1 to 12 and a day to the length of its month; the calendar runs from year 1 to 9999
> let quarters = frame(year = [2025, 2025, 2026], qtr = [3, 4, 1], sales = [10, 12, 9])
> quarters.add(period = date(year, qtr * 3 - 2))
frame · 3 rows × 4 variables

      year   qtr   sales   period
  1   2025     3      10   2025-07-01
  2   2025     4      12   2025-10-01
  3   2026     1       9   2026-01-01
