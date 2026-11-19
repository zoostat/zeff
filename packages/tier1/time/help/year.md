# year

## Summary
Year of a date

## Description
The year of a date, or of a date and time, as a whole number from 1 to 9999. Given a column, `year` applies row by row.

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

> year("2026-10-16")
error[Z0401]: `d` of `year` must be a Date or DateTime, but this is a Str
  = help: usage: `year(d)`

Read the text first: `year(date("2026-10-16"))` is 2026.

## Coming from R or Stata
| year(d) | lubridate::year(d) | year(d) |

Base R has `as.integer(format(d, "%Y"))`. Stata's `year()` takes a date only, and a date and
time must be turned into one first, `year(dofc(t))`; here `year` takes either.

## Related tasks
- For the first day of a date's year, as a date, use `start_of`.
- For whole years between two dates, as an age is counted, use `time_between`.
- To make a date from a year and a month given as numbers, use `date`.
- For the other parts of a date, use `month`, `day` or `quarter`.

## See also
month, day, quarter, week, weekday, date

## Examples
// The year of a date.
> year(date("2026-10-16"))
2026

// Of a date and time: the last second of 1999.
> year(datetime("1999-12-31T23:59:59"))
1999

// Row by row on a column; the missing day stays missing.
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29"), NA])
> sales.add(y = year(sold))
frame · 3 rows × 2 variables

      sold            y
  1   2026-01-05   2026
  2   2024-02-29   2024
  3   NA             NA

// The shipped firms by the decade they were founded in.
> dataset(firms).add(decade = year(founded) - mod(year(founded), 10)).aggregate(firms = n(), by: decade)
frame · 6 rows × 2 variables

      decade   firms
  1     1950      66
  2     1960      71
  3     1970      77
  4     1980      63
  5     1990      70
  6     2000      53
