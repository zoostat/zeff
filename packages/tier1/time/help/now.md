# now

## Summary
Current date and time, the machine's local wall-clock time with no zone

## Description
The machine's local wall-clock time, the date and time a clock on the wall shows where the script
runs, as a `DateTime` with no zone (section 3.8 of the Language Reference). With `today`, it is
the only function whose value depends on when a script runs, so each use is noted (W0703) with
the value, and `zeff run --replay` reuses the recorded value once run records arrive with release
0.11.0; a statement that depends on it is never served from the result cache. It may be called
in a script only (Z0110), never inside a function. Like R's `Sys.time()`, which carries the
session's zone, and Stata's `clock(c(current_date) + " " + c(current_time), "DMYhms")`.

## Arguments

## Value
A `DateTime`.

## Details
Exact: the clock, read once at each call, to the microsecond where the machine gives it. The
help pages' examples and the conformance cases run under a fixed clock, 2026-11-01 09:30:00.

## See also
today, datetime, days_between

## Examples
> now()
2026-11-01T09:30:00
note[W0703]: now() read the clock: 2026-11-01T09:30:00 (line 1)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)
> to(now(), Date)
2026-11-01
note[W0703]: now() read the clock: 2026-11-01T09:30:00 (line 2)
  = help: the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)
