# weekday

## Summary
Day of the week, 1 (Monday) to 7 (Sunday)

## Description
The day of the week of a date, or of a date and time, numbered as ISO 8601 numbers it: Monday 1
to Sunday 7; `NA` gives `NA`. Given a column, `weekday` applies row by row. Like
`lubridate::wday(x, week_start = 1)`; R's `wday` without `week_start` and Stata's `dow()`
start on Sunday, as 1 and 0.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int from 1 to 7; a column of Ints when `d` is a column.

## Details
Exact: 1970-01-01 was a Thursday, and the day of the week advances by one per day in integer
arithmetic.

## See also
week, day, date

## Examples
> weekday(date("2026-10-16"))
5
> weekday(date("2026-10-18"))
7
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29")])
> sales.add(wd = weekday(sold))
frame · 2 rows × 2 variables

      sold         wd
  1   2026-01-05    1
  2   2024-02-29    4
