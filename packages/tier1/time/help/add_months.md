# add_months

## Summary
Date n months later, clamped to the last day of the target month

## Description
The date `n` months after `d`, or before it for a negative `n`, keeping the day of the month
where the target month has it and otherwise taking that month's last day: 31 January plus one
month is 28 February, or 29 in a leap year, as Stata and SQL do and as `lubridate`'s `%m+%` does;
R's `seq(d, by = "month")` rolls over into March instead. On a date and time the time of day is
kept. A result outside the calendar is `NA` with a warning (W0203); `NA` in either argument
gives `NA`. Given a column, `add_months` applies row by row.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.
- n: The months to add, a whole number, negative to go back.

## Value
A value of the type of `d`; a column when `d` is a column.

## Details
Exact: the year and month are moved in integer arithmetic and the day clamped; the operation is
not reversible at a clamped day, since 28 February less one month is 28 January.

## See also
add_days, months_between, date

## Examples
> add_months(date("2026-01-31"), 1)
2026-02-28
> add_months(date("2024-01-31"), 1)
2024-02-29
> add_months(date("2026-10-16"), -12)
2025-10-16
> add_months(date("2026-01-31"), 3)
2026-04-30
> let sales = frame(sold = [date("2026-01-31"), date("2024-02-29")])
> sales.add(renewal = add_months(sold, 12))
frame · 2 rows × 2 variables

      sold         renewal
  1   2026-01-31   2027-01-31
  2   2024-02-29   2025-02-28
