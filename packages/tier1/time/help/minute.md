# minute

## Summary
Minute of a date and time, 0 to 59

## In brief
- `minute(dt)`: the minute of the hour, 0 to 59.
- `visits.add(h = minute(arrived))`: the same for every row of a column of times.

## Description
The minute of the hour of a date and time, as a whole number from 0 to 59; the seconds are dropped, never rounded, so 09:30:59 is minute 30. Given a column, `minute` applies row by row.

## Arguments
- dt: A `DateTime`, or a column of them.

## Value
An Int from 0 to 59; a column of Ints when `dt` is a column.

## Missing values
`NA` gives `NA` for that row, without a note. Nothing is dropped.

## Details
### Times only
`minute` takes a `DateTime` and refuses a `Date`: a date has no time of day, and the engine does
not put midnight in its place without being asked. `to(d, DateTime)` makes a date the midnight
that starts it.

### The clock read
A `DateTime` carries no time zone in 1.0, so the minute is read from the value as written; text
that came with an offset was moved to UTC when it was read, with a warning (W0702).

### Exactness
Read from the microseconds of the time of day in integer arithmetic.

## A common mistake
Asking for the minute of a date. A `Date` holds a day and nothing of the clock:

> minute(date("2026-10-16"))
error[Z0401]: `dt` of `minute` must be a DateTime, but this is a Date
  = help: usage: `minute(dt)`

Read the text with `datetime`, or write `to(d, DateTime)` where midnight is meant.

## Coming from R or Stata
| minute(dt) | lubridate::minute(dt) | mm(dt) |

R's base form is `as.integer(format(dt, "%M"))`. Minutes since midnight are `hour(dt) * 60 + minute(dt)`.

## Related tasks
- For the other parts of the clock, use `hour` and `second`.
- For the day, month or year a time falls on, use `day`, `month` or `year`.
- For whole hours, minutes or seconds between two times, use `time_between`.
- To write a time as text in a pattern of your own, use `format`.

## See also
hour, second, datetime, time_between, format

## Examples
// The minute of the hour.
> minute(datetime("2026-10-16T09:30:07"))
30

// Seconds are dropped, never rounded up.
> minute(datetime("2026-10-16T09:30:59.9"))
30

// Row by row on a column, here as minutes since midnight.
> let visits = frame(arrived = [datetime("2026-03-02T09:15"), datetime("2026-03-02T17:40"), NA])
> visits.add(since_midnight = hour(arrived) * 60 + minute(arrived))
frame · 3 rows × 2 variables

      arrived               since_midnight
  1   2026-03-02T09:15:00              555
  2   2026-03-02T17:40:00             1060
  3   NA                                NA
