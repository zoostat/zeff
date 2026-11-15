# second

## Summary
Whole second of a date and time, 0 to 59

## In brief
- `second(dt)`: the whole second of the minute, 0 to 59.
- `visits.add(h = second(arrived))`: the same for every row of a column of times.

## Description
The second of the minute of a date and time, as a whole number from 0 to 59. A fraction of a second is dropped, never rounded, so 09:30:07.9 is second 7. Given a column, `second` applies row by row.

## Arguments
- dt: A `DateTime`, or a column of them.

## Value
An Int from 0 to 59; a column of Ints when `dt` is a column.

## Missing values
`NA` gives `NA` for that row, without a note. Nothing is dropped.

## Details
### Times only
`second` takes a `DateTime` and refuses a `Date`: a date has no time of day, and the engine does
not put midnight in its place without being asked. `to(d, DateTime)` makes a date the midnight
that starts it.

### The clock read
A `DateTime` carries no time zone in 1.0, so the second is read from the value as written; text
that came with an offset was moved to UTC when it was read, with a warning (W0702). There is no leap second: every minute has 60 seconds.

### Exactness
Read from the microseconds of the time of day in integer arithmetic.

## A common mistake
Asking for the second of a date. A `Date` holds a day and nothing of the clock:

> second(date("2026-10-16"))
error[Z0401]: `dt` of `second` must be a DateTime, but this is a Date
  = help: usage: `second(dt)`

Read the text with `datetime`, or write `to(d, DateTime)` where midnight is meant.

## Coming from R or Stata
| second(dt) | as.integer(lubridate::second(dt)) | floor(ss(dt)) |

`lubridate::second` and Stata's `ss` keep the fraction of the second; `second` gives the whole second, an Int like `hour` and `minute`. For the fraction, `format(dt, pattern: "%S")` writes the seconds as the value holds them.

## Related tasks
- For the other parts of the clock, use `hour` and `minute`.
- For the day, month or year a time falls on, use `day`, `month` or `year`.
- For whole hours, minutes or seconds between two times, use `time_between`.
- To write a time as text in a pattern of your own, use `format`.

## See also
hour, minute, datetime, time_between, format

## Examples
// The second of the minute.
> second(datetime("2026-10-16T09:30:07"))
7

// A fraction of a second is dropped, never rounded up.
> second(datetime("2026-10-16T09:30:07.9"))
7

// Row by row on a column, NA where the time is missing.
> let events = frame(at = [datetime("2026-03-02T09:15:42"), datetime("2026-03-02T17:40:05"), NA])
> events.add(s = second(at))
frame · 3 rows × 2 variables

      at                     s
  1   2026-03-02T09:15:42   42
  2   2026-03-02T17:40:05    5
  3   NA                    NA
