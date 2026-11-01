# day

## Summary
Day of the month

## Description
The day of the month of a date, or of a date and time, as a whole number from 1 to 31; `NA`
gives `NA`. Given a column, `day` applies row by row. Like `lubridate::day()` and Stata's
`day()`.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int; a column of Ints when `d` is a column.

## Details
Exact: read from the calendar day in integer arithmetic.

## See also
year, month, weekday, date

## Examples
> day(date("2026-10-16"))
16
> day(datetime("2024-02-29T12:00"))
29
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29")])
> sales.add(d = day(sold))
frame · 2 rows × 2 variables

      sold          d
  1   2026-01-05    5
  2   2024-02-29   29
