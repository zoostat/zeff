# weekday

## Summary
Day of the week, 1 (Monday) to 7 (Sunday)

## Description
The day of the week of a date, or of a date and time, numbered as ISO 8601 numbers it: Monday 1 to Sunday 7. Given a column, `weekday` applies row by row.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int from 1 to 7; a column of Ints when `d` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact: 1970-01-01 was a Thursday, and the day of the week advances by one per day in integer
arithmetic.

## A common mistake
Giving the text of a date. Text is not read as a date unless that is asked for, since `05/02/2026` is one day in London and another in New York:

> weekday("2026-10-16")
error[Z0401]: `d` of `weekday` must be a Date or DateTime, but this is a Str
  = help: usage: `weekday(d)`

Read the text first: `weekday(date("2026-10-16"))` is 5, a Friday.

## Coming from R or Stata
| weekday(d) | lubridate::wday(d, week_start = 1) | cond(dow(d) == 0, 7, dow(d)) |

Base R has `as.integer(format(d, "%u"))`. `lubridate::wday` without `week_start` numbers the
days from Sunday 1 to Saturday 7, and Stata's `dow()` from Sunday 0 to Saturday 6. Here Monday
is always 1 and Sunday 7, so that `weekday(d) >= 6` is the weekend. R's `weekdays(d)` gives the
name as text, which here is `format(d, pattern: "%A")`.

## Related tasks
- For the Monday that starts a date's week, use `start_of`.
- For the number of the week in the year, use `week`.
- For the day of the month, use `day`.
- For the name of the day, use `format`.

## See also
week, day, date

## Examples
// 16 October 2026 is a Friday.
> weekday(date("2026-10-16"))
5

// Sunday is 7, the last day of the week.
> weekday(date("2026-10-18"))
7

// The weekend is days 6 and 7.
> weekday(date("2026-10-17")) >= 6
true

// Row by row on a column; the missing day stays missing.
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29"), NA])
> sales.add(wd = weekday(sold))
frame · 3 rows × 2 variables

      sold         wd
  1   2026-01-05    1
  2   2024-02-29    4
  3   NA           NA

// The shipped firms by the day of the week they were founded on.
> dataset(firms).add(wd = weekday(founded)).aggregate(firms = n(), by: wd)
frame · 7 rows × 2 variables

      wd   firms
  1    1      65
  2    2      57
  3    3      61
  4    4      45
  5    5      66
  6    6      53
  7    7      53

// The name of the day is a matter of writing the date.
> format(date("2026-10-16"), pattern: "%A")
Friday
