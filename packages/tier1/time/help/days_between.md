# days_between

## Summary
Days from start to end

## Description
The number of days from `start` to `end`, negative when `end` is earlier, the same as `end -
start` on two dates. On two dates and times it counts the whole days between the two instants,
truncated towards zero, so 23 hours apart is 0 days and 47 hours is 1; this is the only
difference two times have in 1.0, since `DateTime - DateTime` is an error until a Duration type
arrives (section 3.8 of the Language Reference). `NA` in either argument gives `NA`. Given
columns, `days_between` applies row by row. Like R's `difftime(end, start, units = "days")`,
truncated, and Stata's `end - start`.

## Arguments
- start: A `Date` or a `DateTime`, or a column of either.
- end: A value of the same type as `start`, or a column of it.

## Value
An Int; a column of Ints when either argument is a column.

## Details
Exact: integer arithmetic on the day counts, or on the microseconds of two times.

## See also
months_between, add_days, date

## Examples
> days_between(date("2026-01-31"), date("2026-03-01"))
29
> days_between(date("2026-03-01"), date("2026-01-31"))
-29
> days_between(datetime("2026-01-31T23:00"), datetime("2026-02-02T22:59"))
1
> let loans = frame(issued = [date("2026-01-05"), date("2026-02-20")], repaid = [date("2026-03-01"), date("2026-02-20")])
> loans.add(open = days_between(issued, repaid))
frame · 2 rows × 3 variables

      issued       repaid       open
  1   2026-01-05   2026-03-01     55
  2   2026-02-20   2026-02-20      0
