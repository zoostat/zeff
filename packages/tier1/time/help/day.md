# day

## Summary
Day of the month

## Description
The day of the month of a date, or of a date and time, as a whole number from 1 to 31. Given a column, `day` applies row by row.

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

> day("2026-10-16")
error[Z0401]: `d` of `day` must be a Date or DateTime, but this is a Str
  = help: usage: `day(d)`

Read the text first: `day(date("2026-10-16"))` is 16.

## Coming from R or Stata
| day(d) | lubridate::day(d) | day(d) |

Base R has `as.integer(format(d, "%d"))`. In all three the name means the day of the month.
The day of the week is `weekday` here, and the day of the year, `lubridate::yday(d)` in R and
`doy(d)` in Stata, is `time_between(start_of(d, year), d) + 1`.

## Related tasks
- For the day of the week, use `weekday`.
- For whole days between two dates, use `time_between`.
- For the date some days later, use `add_time`.
- For the first day of a date's month, use `start_of`.

## See also
year, month, weekday, date

## Examples
// The day of the month.
> day(date("2026-10-16"))
16

// Of a date and time, on the day a leap year adds.
> day(datetime("2024-02-29T12:00"))
29

// Row by row on a column; the missing day stays missing.
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29"), NA])
> sales.add(d = day(sold))
frame · 3 rows × 2 variables

      sold          d
  1   2026-01-05    5
  2   2024-02-29   29
  3   NA           NA

// The one shipped firm founded on a 29 February.
> dataset(firms).where(month(founded) == 2 && day(founded) == 29)
frame · 1 row × 4 variables

      firm   industry   size   founded
  1    246   Services     19   1992-02-29

// The day of the year: 16 October 2026 is its 289th day.
> let d = date("2026-10-16")
> time_between(start_of(d, year), d) + 1
289
