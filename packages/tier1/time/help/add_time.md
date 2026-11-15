# add_time

## Summary
Date or time a number of days, weeks, months or years later

## In brief
- `add_time(due, 30)`: the date 30 days later; a negative number goes back.
- `add_time(signed, 6, unit: month)`: six months later, on the same day of the month.
- `add_time(born, 18, unit: year)`: a birthday; `unit: week` moves by sevens of days.

## Description
Moves a date, or a date and time, by `n` whole units and gives a value of the same type: a
`Date` stays a `Date`, and a `DateTime` keeps its time of day. `n` is a whole number, positive
to go forward and negative to go back; 0 gives `d` itself.

A day and a week have a fixed length. A month does not, so a move by months keeps the day of
the month and, where the target month is too short, takes its last day: 31 January plus one
month is 28 February, or 29 in a leap year. A year is twelve such months. Given columns,
`add_time` applies row by row, and `n` may be a column too.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.
- n: How many units to move, a whole number, negative to go back; or an Int column.
- unit: What `n` counts: `day`, `week`, `month` or `year`; `day` unless given.

## Value
A value of the type of `d`; a column when `d` or `n` is a column.

## Missing values
`NA` in `d` or in `n` gives `NA` for that row, without a note. A result outside the calendar,
years 1 to 9999, is `NA` with a warning (W0203) that counts the rows.

## Details
### Month-end clamping
The day is clamped to the last day of the target month, as Stata's and SQL's month arithmetic
and `lubridate`'s `%m+%` do; R's `seq(d, by = "month")` rolls over into the next month instead,
so that 31 January plus one month is 3 March. Because of the clamping a move by months does not
always undo itself: 31 January plus one month is 28 February, and 28 February less one month
is 28 January.

### Counting back the same way
`time_between(start, end, unit: month)` counts months by this rule, so
`time_between(d, add_time(d, n, unit: month), unit: month)` is `n`.

### Exactness
Integer arithmetic on the calendar; a `DateTime` carries no time zone in 1.0, so a day is
always 24 hours and the time of day never shifts.

## A common mistake
Adding a number to a date with `+`. A number beside a date could mean days, months or years, so
the engine asks which:

> date("2026-01-31") + 30
error[Z0221]: `+` does not apply to a Date and a number
  = help: `add_time(d, n)` moves a date by n days, and by weeks, months or years with `unit:`; `Date - Date` gives the days between two dates

Write `add_time(date("2026-01-31"), 30)`.

## Coming from R or Stata
| add_time(d, n) | d + n | d + n |
| add_time(d, n, unit: week) | d + 7 * n | d + 7 * n |
| add_time(d, n, unit: month) | d %m+% months(n) | none built in |
| add_time(d, n, unit: year) | d %m+% years(n) | none built in |

`%m+%` is `lubridate`'s; plain `d + months(n)` there gives `NA` at a month's end where this
function clamps. Stata adds months on a monthly date, `mofd(d) + n`, and `dofm` of that is the
first day of the target month, not the same day of it.

## Related tasks
- To count the days, weeks, months or years between two dates, use `time_between`.
- For the first day of the month, quarter or year a date falls in, use `start_of`.
- For today's date to move from, use `today`.
- To build a date from a year, a month and a day held as numbers, use `date`.

## See also
time_between, start_of, date, today

## Examples
// Thirty days later; the unit is the day unless given.
> add_time(date("2026-01-31"), 30)
2026-03-02

// A negative number goes back.
> add_time(date("2026-01-31"), -31)
2025-12-31

// A month later keeps the day of the month, or takes the month's last day.
> add_time(date("2026-01-31"), 1, unit: month)
2026-02-28

// In a leap year that last day is the 29th.
> add_time(date("2024-01-31"), 1, unit: month)
2024-02-29

// Two weeks are fourteen days.
> add_time(date("2026-10-16"), 2, unit: week)
2026-10-30

// A year later from 29 February is 28 February.
> add_time(date("2024-02-29"), 1, unit: year)
2025-02-28

// A time keeps its time of day.
> add_time(datetime("2026-01-31T09:30:00"), 1, unit: month)
2026-02-28T09:30:00

// Row by row on columns: each contract's renewal date, the term held in months.
> let contracts = frame(signed = [date("2026-01-31"), date("2024-02-29"), NA], term = [12, 6, 12])
> contracts.add(renewal = add_time(signed, term, unit: month))
frame · 3 rows × 3 variables

      signed       term   renewal
  1   2026-01-31     12   2027-01-31
  2   2024-02-29      6   2024-08-29
  3   NA             12   NA
