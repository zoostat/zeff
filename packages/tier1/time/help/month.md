# month

## Summary
Month of a date, 1 to 12

## Description
The month of a date, or of a date and time, as a whole number, January 1 to December 12; `NA`
gives `NA`. Given a column, `month` applies row by row. Like `lubridate::month()` and Stata's
`month()`.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int; a column of Ints when `d` is a column.

## Details
Exact: read from the calendar day in integer arithmetic.

## See also
year, day, quarter, week, date

## Examples
> month(date("2026-10-16"))
10
> month(datetime("2026-02-01T00:00"))
2
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29")])
> sales.add(m = month(sold))
frame · 2 rows × 2 variables

      sold         m
  1   2026-01-05   1
  2   2024-02-29   2
