# hour

## Summary
Hour of a date and time, 0 to 23

## In brief
- `hour(dt)`: the hour of the day, 0 for the hour after midnight and 23 for the last.
- `visits.add(h = hour(arrived))`: the same for every row of a column of times.

## Description
The hour of the day of a date and time, as a whole number on the 24-hour clock: 0 from midnight to 00:59:59, 13 from one in the afternoon, 23 for the last hour of the day. Given a column, `hour` applies row by row.

## Arguments
- dt: A `DateTime`, or a column of them.

## Value
An Int from 0 to 23; a column of Ints when `dt` is a column.

## Missing values
`NA` gives `NA` for that row, without a note. Nothing is dropped.

## Details
### Times only
`hour` takes a `DateTime` and refuses a `Date`: a date has no time of day, and the engine does
not put midnight in its place without being asked. `to(d, DateTime)` makes a date the midnight
that starts it.

### The clock read
A `DateTime` carries no time zone in 1.0, so the hour is read from the value as written; text
that came with an offset was moved to UTC when it was read, with a warning (W0702).

### Exactness
Read from the microseconds of the time of day in integer arithmetic.

## A common mistake
Asking for the hour of a date. A `Date` holds a day and nothing of the clock:

> hour(date("2026-10-16"))
error[Z0401]: `dt` of `hour` must be a DateTime, but this is a Date
  = help: usage: `hour(dt)`

Read the text with `datetime`, or write `to(d, DateTime)` where midnight is meant.

## Coming from R or Stata
| hour(dt) | lubridate::hour(dt) | hh(dt) |

R's base form is `as.integer(format(dt, "%H"))`. Both read the clock in the time zone the value carries; Zeff reads the value as it stands, which for text read with an offset is UTC.

## Related tasks
- For the other parts of the clock, use `minute` and `second`.
- For the day, month or year a time falls on, use `day`, `month` or `year`.
- For whole hours, minutes or seconds between two times, use `time_between`.
- To write a time as text in a pattern of your own, use `format`.

## See also
minute, second, datetime, time_between, format

## Examples
// The hour on the 24-hour clock.
> hour(datetime("2026-10-16T09:30:07"))
9

// One in the afternoon is 13.
> hour(datetime("2026-10-16T13:05"))
13

// The last second of a day is in hour 23.
> hour(datetime("1999-12-31T23:59:59"))
23

// Row by row on a column: arrivals by hour of the day, NA where the time is missing.
> let visits = frame(arrived = [datetime("2026-03-02T09:15"), datetime("2026-03-02T17:40"), NA])
> visits.add(h = hour(arrived))
frame · 3 rows × 2 variables

      arrived                h
  1   2026-03-02T09:15:00    9
  2   2026-03-02T17:40:00   17
  3   NA                    NA
