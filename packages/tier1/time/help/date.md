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

## Arguments
- s: The text, or a column of text.
- format: The format the text is written in, when it is not ISO 8601.

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
warning: text that is not a date in ISO 8601 became NA (line 4)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`
> let sales = frame(sold = ["2026-01-05", "5/2/2026", "2026-03-01"])
> sales.add(d = date(sold))
frame · 3 rows × 2 variables

      sold         d
  1   2026-01-05   2026-01-05
  2   5/2/2026     NA
  3   2026-03-01   2026-03-01
warning: text that is not a date in ISO 8601 became NA in 1 row (line 6)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`
