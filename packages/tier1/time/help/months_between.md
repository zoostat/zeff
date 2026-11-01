# months_between

## Summary
Whole months from start to end, with month-end clamping

## Description
The number of whole months from `start` to `end`: the largest `m` such that `add_months(start,
m)` is at or before `end`, so that 31 January to 28 February is one month, as the clamping of
`add_months` has it, and 31 January to 27 February none; negative when `end` is earlier, counted
the same way backwards. On dates and times the days alone count. `NA` in either argument gives
`NA`. Given columns, `months_between` applies row by row. Like `lubridate`'s
`interval(start, end) %/% months(1)`; Stata's `mofd(end) - mofd(start)` counts calendar months
instead, 31 January to 1 February being one.

## Arguments
- start: A `Date` or a `DateTime`, or a column of either.
- end: A value of the same type as `start`, or a column of it.

## Value
An Int; a column of Ints when either argument is a column.

## Details
Exact: integer arithmetic on the years, months and days, with the clamping of `add_months`.

## See also
days_between, add_months, date

## Examples
> months_between(date("2026-01-31"), date("2026-02-28"))
1
> months_between(date("2026-01-31"), date("2026-02-27"))
0
> months_between(date("2026-03-31"), date("2026-01-31"))
-2
> months_between(date("2020-02-29"), date("2026-02-28"))
72
