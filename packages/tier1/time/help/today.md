# today

## Summary
Current date, read from the clock and noted

## Description
The date of the machine's clock, the day a clock on the wall shows where the script runs, with no
zone. With `now`, it is the only function whose value depends on when a script runs, so each use
is noted (W0703) with the value, and `zeff run --replay` reuses the recorded value once run
records arrive with release 0.11.0; a statement that depends on it is never served from the
result cache. It may be called in a script only (Z0110), never inside a function, so that a
function's result depends on its arguments alone. Like R's `Sys.Date()` and Stata's `today()`.

## Arguments

## Value
A `Date`.

## Details
Exact: the day of the clock, read once at each call. The help pages' examples and the conformance
cases run under a fixed clock, 2026-11-01 09:30:00, so that they print one value everywhere.

## See also
now, date, days_between

## Examples
> today()
2026-11-01
note[W0703]: today() read the clock: 2026-11-01 (line 1)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)
> days_between(today(), date("2027-01-01"))
61
note[W0703]: today() read the clock: 2026-11-01 (line 2)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)
