# start_of

## Summary
Start of the hour, day, week, month, quarter or year a date falls in

## In brief
- `start_of(d, month)`: the first day of the month of `d`; `quarter` and `year` likewise.
- `start_of(d, week)`: the Monday of the week of `d`.
- `start_of(t, hour)`: a time with its minutes and seconds set to zero; `day` gives midnight.

## Description
Moves a date, or a date and time, back to the start of the period it lies in, and gives a value
of the same type: a `Date` stays a `Date`, and a `DateTime` becomes the first instant of the
period, at 00:00:00 for every unit but `hour`. A value already at the start is unchanged.

It is the way to group days into months, quarters or weeks and keep a date that sorts, plots on
a date axis and can be counted from: `visits.add(month = start_of(day, month))` and then
`aggregate(..., by: month)`. Given a column, `start_of` applies row by row.

## Arguments
- d: A `Date` or a `DateTime`, or a column of either.
- unit: The period, as a bare word: `hour`, `day`, `week`, `month`, `quarter` or `year`.

## Value
A value of the type of `d`; a column of them when `d` is a column.

## Missing values
`NA` gives `NA` for that row, without a note. Nothing is dropped.

## Details
### The week
A week starts on Monday, as `weekday` counts its days from 1 on Monday, and as ISO 8601 has it.
The first week of a year may therefore start in the year before: the week of Thursday
1 January 2026 starts on Monday 29 December 2025.

### Quarters
Quarters are those of the calendar year, starting on 1 January, 1 April, 1 July and 1 October,
as `quarter` numbers them.

### Dates and times
`hour` needs a `DateTime`: a `Date` has no time of day, so the start of its hour is refused
rather than taken to be midnight. On a `Date`, `day` gives the date itself.

### Exactness
Computed on the calendar in integer arithmetic; a `DateTime` carries no time zone in 1.0, so
no hour is ever skipped or repeated.

## A common mistake
Writing the unit in quotes. The unit is a word of the language, like `robust` in `se: robust`,
not text:

> start_of(date("2026-10-16"), "month")
error[Z0403]: expected one of hour, day, week, month, quarter, year, found text
  = help: write the word without quotes: `month`

Write `start_of(d, month)`.

## Coming from R or Stata
| start_of(d, month) | lubridate::floor_date(d, "month") | dofm(mofd(d)) |
| start_of(d, quarter) | lubridate::floor_date(d, "quarter") | dofq(qofd(d)) |
| start_of(d, year) | lubridate::floor_date(d, "year") | dofy(yofd(d)) |
| start_of(d, week) | lubridate::floor_date(d, "week", week_start = 1) | d - mod(dow(d) + 6, 7) |
| start_of(t, hour) | lubridate::floor_date(t, "hour") | dhms(dofc(t), hh(t), 0, 0) |

`floor_date` starts a week on Sunday unless `week_start = 1` is given; here a week always starts
on Monday. Stata's `mofd` gives a month number, not a date, which is why `dofm` turns it back;
`start_of` gives the date at once. Base R's `cut(d, "month")` gives a factor of labels, not dates.

## Related tasks
- For the month, quarter or week as a number instead of a date, use `month`, `quarter` or `week`.
- To build a date from a year and a month held as numbers, use `date`.
- To count whole months or weeks between two dates, use `time_between`.
- For the first and the last date of a variable, use `min` and `max`.

## See also
date, month, quarter, week, weekday, time_between

## Examples
// The first day of the month.
> start_of(date("2026-10-16"), month)
2026-10-01

// The Monday of the week: 16 October 2026 is a Friday.
> start_of(date("2026-10-16"), week)
2026-10-12

// The quarter and the year start on the first of January, April, July or October.
> start_of(date("2026-08-31"), quarter)
2026-07-01

// A week may start in the year before.
> start_of(date("2026-01-01"), week)
2025-12-29

// A time keeps its type: the hour with its minutes and seconds at zero.
> start_of(datetime("2026-10-16T09:30:07"), hour)
2026-10-16T09:00:00

// Midnight of the day, and of the first day of the year.
> start_of(datetime("2026-10-16T09:30:07"), day)
2026-10-16T00:00:00

// Row by row on a column: visits counted by month, on dates that sort and plot.
> let visits = frame(day = [date("2026-01-05"), date("2026-01-28"), date("2026-02-03"), NA], cost = [40, 55, 30, 20])
> visits.add(month = start_of(day, month)).aggregate(visits = n(), cost = sum(cost), by: month)
frame · 3 rows × 3 variables

      month        visits   cost
  1   2026-01-01        2     95
  2   2026-02-01        1     30
  3   NA                1     20
