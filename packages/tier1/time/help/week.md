# week

## Summary
ISO week number

## Description
The week of the year a date, or a date and time, falls in, as ISO 8601 numbers weeks: a week
runs from Monday to Sunday and belongs to the year that holds its Thursday, so week 1 is the
week of 4 January, the first days of January may fall in week 52 or 53 of the year before, and
the last days of December in week 1 of the year after; `NA` gives `NA`. Given a column, `week`
applies row by row. Like `lubridate::isoweek()`; R's `week()` and Stata's `week()` count blocks
of seven days from 1 January instead, which gives other numbers.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int from 1 to 53; a column of Ints when `d` is a column.

## Details
Exact: the Thursday of the week is found in integer arithmetic and the week counted from the
first day of its year.

## References
ISO 8601-1:2019, Date and time — Representations for information interchange, section 4.2.2
(week calendar).

## See also
weekday, year, quarter, date

## Examples
> week(date("2026-10-16"))
42
> week(date("2021-01-01"))
53
> week(date("2021-01-04"))
1
> let sales = frame(sold = [date("2024-12-30"), date("2024-12-29")])
> sales.add(w = week(sold))
frame · 2 rows × 2 variables

      sold          w
  1   2024-12-30    1
  2   2024-12-29   52
