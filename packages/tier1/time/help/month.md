# month

## Summary
Month of a date, 1 to 12

## Description
The month of a date, or of a date and time, as a whole number: January 1 to December 12. Given a column, `month` applies row by row.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int; a column of Ints when `d` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact: read from the calendar day in integer arithmetic.

## A common mistake
Giving the text of a date. Text is not read as a date unless that is asked for, since `05/02/2026` is one day in London and another in New York:

> month("2026-10-16")
error[Z0401]: `d` of `month` must be a Date or DateTime, but this is a Str
  = help: usage: `month(d)`

Read the text first: `month(date("2026-10-16"))` is 10.

## Coming from R or Stata
| month(d) | lubridate::month(d) | month(d) |

Base R has `as.integer(format(d, "%m"))`; its `months(d)` gives the name of the month as text,
in the language of the computer. Stata's `month()` takes a date only, and a date and time needs
`month(dofc(t))`. Here the number is always given, and the name is a matter of writing the
date: `format(d, pattern: "%B")`.

## Related tasks
- For the first day of a date's month, as a date that sorts and plots, use `start_of`.
- For whole months between two dates, use `time_between`.
- For the date some months later, use `add_time`.
- For the name of the month, use `format`.

## See also
year, day, quarter, week, date

## Examples
// The month of a date.
> month(date("2026-10-16"))
10

// Of a date and time.
> month(datetime("2026-02-01T00:00"))
2

// Row by row on a column; the missing day stays missing.
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29"), NA])
> sales.add(m = month(sold))
frame · 3 rows × 2 variables

      sold          m
  1   2026-01-05    1
  2   2024-02-29    2
  3   NA           NA

// The shipped firms founded in each of the first three months.
> dataset(firms).add(m = month(founded)).aggregate(firms = n(), by: m).head(3)
frame · 3 rows × 2 variables

      m   firms
  1   1      35
  2   2      30
  3   3      27

// The name of the month is a matter of writing the date.
> format(date("2026-10-16"), pattern: "%B")
October
