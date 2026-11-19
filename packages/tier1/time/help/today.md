# today

## Summary
Current date, read from the clock and noted

## Description
The date of the machine's clock, the day a clock on the wall shows where the script runs, with no
zone. With `now`, it is the only function whose value depends on when a script runs, so each use
is noted (W0703) with the value, and `zeff run --replay` reuses the recorded value once run
records arrive with release 0.11.0; a statement that depends on it is never served from the
result cache. It may be called in a script only (Z0110), never inside a function, so that a
function's result depends on its arguments alone.

## Arguments


## Value
A `Date`.

## Details
Exact: the day of the clock, read once at each call. The help pages' examples and the conformance
cases run under a fixed clock, 2026-11-01 09:30:00, so that they print one value everywhere.

## A common mistake
Reading the clock inside a function. A function's result must follow from its arguments alone, so that it can be kept and reused; one that read the clock would give another answer tomorrow:

> /// Whole years from a birthday to today.
> fn age_today(born: Date) -> Int { time_between(born, today(), unit: year) }
error[Z0110]: `today` is allowed only in a script, not inside the function `age_today`
  = help: a function's result depends on its arguments alone, so it may be cached; import the data in the script and pass the frame in

Read the clock in the script and pass the day in: write the function as `fn age_on(born: Date, on: Date) -> Int { time_between(born, on, unit: year) }` and call it as `age_on(born, today())`.

## Coming from R or Stata
| today() | Sys.Date() | date(c(current_date), "DMY") |

R and Stata read the clock without a word, so a result that depends on the day the script ran
looks like any other. Here each reading is noted with the value read, and a function may not
read the clock at all.

## Related tasks
- For the time of day as well, use `now`.
- For a fixed day, which gives the same result on every run, use `date`.
- For whole days, months or years from a date to today, use `time_between`.

## See also
now, date, time_between

## Examples
// The day of the clock; the note records the value read.
> today()
2026-11-01
note[W0703]: today() read the clock: 2026-11-01 (line 1)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)

// Whole days from today to a date.
> time_between(today(), date("2027-01-01"))
61
note[W0703]: today() read the clock: 2026-11-01 (line 2)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)

// Bound to a name, the clock is read once and the day used three times.
> let on = today()
> [year(on), month(on), day(on)]
note[W0703]: today() read the clock: 2026-11-01 (line 3)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)
2026
11
1

// The age of the shipped firms in whole years, as of today.
> dataset(firms).head(3).add(age = time_between(founded, today(), unit: year))
frame · 3 rows × 5 variables

      firm   industry      size   founded      age
  1      1   Agriculture      3   1990-11-06    35
  2      2   Agriculture      5   1957-07-02    69
  3      3   Agriculture      3   1998-11-07    27
note[W0703]: today() read the clock: 2026-11-01 (line 5)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)
