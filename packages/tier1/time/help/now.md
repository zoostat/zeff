# now

## Summary
Current date and time, the machine's local wall-clock time with no zone

## Description
The machine's local wall-clock time, the date and time a clock on the wall shows where the script
runs, as a `DateTime` with no zone (section 3.8 of the Language Reference). With `today`, it is
the only function whose value depends on when a script runs, so each use is noted (W0703) with
the value, and `zeff run --replay` reuses the recorded value once run records arrive with release
0.11.0; a statement that depends on it is never served from the result cache. It may be called
in a script only (Z0110), never inside a function.

## Arguments


## Value
A `DateTime`.

## Details
Exact: the clock, read once at each call, to the microsecond where the machine gives it. The
help pages' examples and the conformance cases run under a fixed clock, 2026-11-01 09:30:00.

## A common mistake
Reading the clock inside a function. A function's result must follow from its arguments alone, so that it can be kept and reused; one that read the clock would give another answer a second later:

> /// Whole seconds from a time to now.
> fn seconds_since(start: DateTime) -> Int { time_between(start, now(), unit: second) }
error[Z0110]: `now` is allowed only in a script, not inside the function `seconds_since`
  = help: a function's result depends on its arguments alone, so it may be cached; import the data in the script and pass the frame in

Read the clock in the script and pass the time in: write the function as `fn seconds_from(start: DateTime, end: DateTime) -> Int { time_between(start, end, unit: second) }` and call it as `seconds_from(start, now())`.

## Coming from R or Stata
| now() | Sys.time() | clock(c(current_date) + " " + c(current_time), "DMYhms") |

R's `Sys.time()` is an instant that prints in the time zone of the session; here the value is
the time the wall clock shows, with no zone, so it prints the same wherever it is read later.
R and Stata read the clock without a word; here each reading is noted with the value read.

## Related tasks
- For the day alone, use `today`.
- For a fixed time, which gives the same result on every run, use `datetime`.
- For whole seconds, minutes or hours from a time to now, use `time_between`.
- For the hour of the day, use `hour`.

## See also
today, datetime, time_between

## Examples
// The time of the clock; the note records the value read.
> now()
2026-11-01T09:30:00
note[W0703]: now() read the clock: 2026-11-01T09:30:00 (line 1)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)

// The day it falls on.
> to(now(), Date)
2026-11-01
note[W0703]: now() read the clock: 2026-11-01T09:30:00 (line 2)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)

// Whole minutes since a time this morning.
> time_between(datetime("2026-11-01T08:00"), now(), unit: minute)
90
note[W0703]: now() read the clock: 2026-11-01T09:30:00 (line 3)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)

// Bound to a name, the clock is read once and the time used twice.
> let at = now()
> [hour(at), minute(at)]
note[W0703]: now() read the clock: 2026-11-01T09:30:00 (line 4)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)
9
30
