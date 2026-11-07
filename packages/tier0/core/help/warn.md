# warn

## Summary
Print a warning and continue

## Description
Prints `message` as a warning under the statement's output, with the line it came from, and carries
on: the way for a script to flag a condition its author wants noticed without stopping the run, such
as a small sample. The warning is printed as the engine's own are (section 10.2 of the Language
Reference) and is kept in the run record when that exists (release 0.11.0). With `code:` the
warning carries a warning's code, `warning[W0205]: ...`, so that a function written in Zeff can
raise the warning of section 11 that its page declares and a pragma `// zeff: allow(CODE) reason`
can silence it; a code no warning has is an error (Z0401).
`warn` returns nothing: it is a statement, and its result cannot be bound or passed on (Z0206).
R has `warning`.

## Arguments
- message: The text to print, a Str.
- code: A warning's code to print the warning under, `W` and four digits, written bare (`code: W0205`) or as text; without it the warning carries no code.

## Value
Nothing.

## Details
Exact: the text is printed as given, after the word `warning:` (or `warning[CODE]:` with a code),
and the line number follows in parentheses. `--deny-warnings` refuses a warning of `warn` with or
without a code (section 11.1).

## References
Language Reference, section 5.4 ("Errors, warnings and failed draws").

## See also
note, error, print

## Examples
> warn("fewer than 30 rows: treat large-sample tests with care")
warning: fewer than 30 rows: treat large-sample tests with care (line 1)
> warn("the standard deviation of one value is NA", code: W0205)
warning[W0205]: the standard deviation of one value is NA (line 2)
> print("the run carries on")
the run carries on
