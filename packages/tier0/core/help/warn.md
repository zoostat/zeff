# warn

## Summary
Print a warning and continue

## Description
Prints `message` as a warning under the statement's output, with the line it came from, and carries
on: the way for a script to flag a condition its author wants noticed without stopping the run, such
as a small sample. The warning is printed as the engine's own are (section 10.2 of the Language
Reference) and is kept in the run record when that exists (release 0.11.0). `warn` returns nothing:
it is a statement, and its result cannot be bound or passed on (Z0206). R has `warning`.

## Arguments
- message: The text to print, a Str.

## Value
Nothing.

## Details
Exact: the text is printed as given, after the word `warning:`, and the line number follows in
parentheses.

## References
Language Reference, section 5.4 ("Errors, warnings and failed draws").

## See also
error, print

## Examples
> warn("fewer than 30 rows: treat large-sample tests with care")
warning: fewer than 30 rows: treat large-sample tests with care (line 1)
> print("the run carries on")
the run carries on
