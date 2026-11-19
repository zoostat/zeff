# week

## Summary
ISO week number

## Description
The week of the year a date, or a date and time, falls in, as ISO 8601 numbers weeks: a week runs from Monday to Sunday and belongs to the year that holds its Thursday, so week 1 is the week of 4 January, the first days of January may fall in week 52 or 53 of the year before, and the last days of December in week 1 of the year after. Given a column, `week` applies row by row.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.

## Value
An Int from 1 to 53; a column of Ints when `d` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact: the Thursday of the week is found in integer arithmetic and the week counted from the
first day of its year.

## A common mistake
Giving the text of a date. Text is not read as a date unless that is asked for, since `05/02/2026` is one day in London and another in New York:

> week("2026-10-16")
error[Z0401]: `d` of `week` must be a Date or DateTime, but this is a Str
  = help: usage: `week(d)`

Read the text first: `week(date("2026-10-16"))` is 42.

## Coming from R or Stata
| week(d) | lubridate::isoweek(d) | no single function |

Base R has `as.integer(format(d, "%V"))`. `lubridate::week()` and Stata's `week()` count blocks
of seven days from 1 January instead, so that 1 January is always in week 1 and the numbers
differ from these in most years: for 1 January 2021 they give 1, and the ISO week is 53. The
year an ISO week belongs to, `lubridate::isoyear(d)` in R, is the year of the week's Thursday:
`year(add_time(start_of(d, week), 3))`.

## Related tasks
- For the Monday that starts a date's week, as a date that sorts and plots, use `start_of`.
- For the day of the week, use `weekday`.
- For whole weeks between two dates, use `time_between`.

## References
ISO 8601-1:2019, Date and time — Representations for information interchange, section 4.2.2
(week calendar).

## See also
weekday, year, quarter, date

## Examples
// The week of 16 October 2026.
> week(date("2026-10-16"))
42

// 1 January 2021, a Friday, is in the last week of 2020.
> week(date("2021-01-01"))
53

// Week 1 of 2021 starts on Monday 4 January.
> week(date("2021-01-04"))
1

// The year that week belongs to: the year of its Thursday.
> year(add_time(start_of(date("2021-01-01"), week), 3))
2020

// Row by row on a column: the last days of December may be in week 1.
> let sales = frame(sold = [date("2024-12-30"), date("2024-12-29"), NA])
> sales.add(w = week(sold))
frame · 3 rows × 2 variables

      sold          w
  1   2024-12-30    1
  2   2024-12-29   52
  3   NA           NA

// The one shipped firm founded in a week 53.
> dataset(firms).where(week(founded) == 53)
frame · 1 row × 4 variables

      firm   industry    size   founded
  1    317   Education      3   1959-12-30
