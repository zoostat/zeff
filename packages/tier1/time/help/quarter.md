# quarter

## Summary
Quarter, 1 to 4

## Description
The quarter of the year a date, or a date and time, falls in: January to March 1, April to June
2, July to September 3, October to December 4; `NA` gives `NA`. Given a column, `quarter`
applies row by row. Like `lubridate::quarter()` and Stata's `quarter()`. A fiscal year that
starts in another month is a `Period` column type of a later release (section 4.3 of the
Language Reference).

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int from 1 to 4; a column of Ints when `d` is a column.

## Details
Exact: read from the month in integer arithmetic.

## See also
month, year, week, date

## Examples
> quarter(date("2026-10-16"))
4
> quarter(date("2026-03-31"))
1
> let sales = frame(sold = [date("2026-01-05"), date("2024-08-29")])
> sales.add(q = quarter(sold))
frame · 2 rows × 2 variables

      sold         q
  1   2026-01-05   1
  2   2024-08-29   3
