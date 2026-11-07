# datetime

## Summary
Parse text into a date and time

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
warning. R's `as.POSIXct` reads the same forms in the session's zone; Zeff reads every value as
UTC and never shifts it afterwards.

## Arguments
- s: The text, or a column of text.
- format: The format the text is written in, when it is not ISO 8601.

## Value
A `DateTime`; a column of them when `s` is a column.

## Details
Exact: seconds are kept to the microsecond and the calendar is that of `date`, so every target
gives the same instant. A `DateTime` prints in ISO 8601 with `T`, `2026-10-16T09:30:00`, and
with its fraction when it has one.

## See also
date, to, year, day

## Examples
> datetime("2026-10-16T09:30")
2026-10-16T09:30:00
> datetime("2026-10-16 09:30:00.25")
2026-10-16T09:30:00.250000
> datetime("2026-10-16")
2026-10-16T00:00:00
> datetime("16/10/2026 09:30", format: "%d/%m/%Y %H:%M")
2026-10-16T09:30:00
> datetime("2026-10-16T09:30:00+02:00")
2026-10-16T07:30:00
warning[W0702]: a time-zone offset was applied to reach UTC (line 5)
  = help: a DateTime has no zone in 1.0; the instant is kept, written in UTC
> datetime("2026-10-16T25:00")
NA
warning[W0203]: text that is not a date and time in ISO 8601 became NA (line 6)
  = help: `to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: "%d/%m/%Y"`
