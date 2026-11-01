# year

## Summary
Year of a date

## Description
The year of a date, or of a date and time, as a whole number from 1 to 9999; `NA` gives `NA`.
Given a column, `year` applies row by row. Like `lubridate::year()` and Stata's `year()`.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int; a column of Ints when `d` is a column.

## Details
Exact: read from the calendar day in integer arithmetic.

## See also
month, day, quarter, week, weekday, date

## Examples
> year(date("2026-10-16"))
2026
> year(datetime("1999-12-31T23:59:59"))
1999
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29")])
> sales.add(y = year(sold))
frame · 2 rows × 2 variables

      sold            y
  1   2026-01-05   2026
  2   2024-02-29   2024
