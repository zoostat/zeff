# quarter

## Summary
Quarter, 1 to 4

## Description
The quarter of the year a date, or a date and time, falls in: January to March 1, April to June 2, July to September 3, October to December 4. Given a column, `quarter` applies row by row. A fiscal year that starts in another month is a `Period` column type of a later release (section 4.3 of the Language Reference).

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int from 1 to 4; a column of Ints when `d` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact: read from the month in integer arithmetic.

## A common mistake
Giving the text of a date. Text is not read as a date unless that is asked for, since `05/02/2026` is one day in London and another in New York:

> quarter("2026-10-16")
error[Z0401]: `d` of `quarter` must be a Date or DateTime, but this is a Str
  = help: usage: `quarter(d)`

Read the text first: `quarter(date("2026-10-16"))` is 4.

## Coming from R or Stata
| quarter(d) | lubridate::quarter(d) | quarter(d) |

Base R's `quarters(d)` gives text, `"Q4"`. For the year and the quarter together Stata has
`qofd(d)`, a count of quarters since 1960 that reads as a quarter only under a display format;
here `start_of(d, quarter)` gives the first day of the quarter as a date, which sorts, plots and
declares the time of a panel.

## Related tasks
- For the first day of a date's quarter, as a date, use `start_of`.
- For the month, use `month`.
- To make a date from a year and a quarter given as numbers, use `date`.

## See also
month, year, week, date

## Examples
// October is in the fourth quarter.
> quarter(date("2026-10-16"))
4

// The last day of March is still in the first.
> quarter(date("2026-03-31"))
1

// Row by row on a column; the missing day stays missing.
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29"), NA])
> sales.add(q = quarter(sold))
frame · 3 rows × 2 variables

      sold          q
  1   2026-01-05    1
  2   2024-02-29    1
  3   NA           NA

// The shipped firms by the quarter they were founded in.
> dataset(firms).add(q = quarter(founded)).aggregate(firms = n(), by: q)
frame · 4 rows × 2 variables

      q   firms
  1   1      92
  2   2      91
  3   3     126
  4   4      91

// The year and the quarter together: the first day of the quarter, a date.
> start_of(date("2026-10-16"), quarter)
2026-10-01
