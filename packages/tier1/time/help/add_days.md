# add_days

## Summary
Date n days later

## Description
The date `n` days after `d`, or before it for a negative `n`; on a date and time, the time of day
is kept. A result outside the calendar, years 1 to 9999, is `NA` with a warning (W0203); `NA` in
either argument gives `NA`. Given a column, `add_days` applies row by row. This is the only way
to move a date by days: `d + 7` is an error (Z0221), since a date is not a number (section 3.7
of the Language Reference). Like R's `d + n` and Stata's `d + n` on daily dates.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.
- n: The days to add, a whole number, negative to go back.

## Value
A value of the type of `d`; a column when `d` is a column.

## Details
Exact: integer arithmetic on the day count.

## See also
add_months, days_between, date

## Examples
> add_days(date("2026-10-16"), 30)
2026-11-15
> add_days(date("2026-03-01"), -1)
2026-02-28
> add_days(datetime("2026-10-16T09:30"), 1)
2026-10-17T09:30:00
> add_days(date("9999-12-31"), 1)
NA
warning: a date outside years 1 to 9999 became NA (line 4)
  = help: the calendar runs from 0001-01-01 to 9999-12-31 (section 3.8 of the Language Reference)
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29")])
> sales.add(due = add_days(sold, 30))
frame · 2 rows × 2 variables

      sold         due
  1   2026-01-05   2026-02-04
  2   2024-02-29   2024-03-30
