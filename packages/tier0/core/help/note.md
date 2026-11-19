# note

## Summary
Print a note and continue

## Description
Prints `message` as a note under the statement's output, with the line it came from, and carries
on: the way for a script, or a function, to say what it did without flagging anything, such as
how many rows a step dropped. A note is printed as the engine's own are (section 10.2 of the
Language Reference), is hidden by `zeff run --quiet`, and is kept in the run record when that
exists (release 0.11.0). With `code:` the note carries a warning's code, `note[W0703]: ...`, so
that a function written in Zeff can raise the note of section 11 that its page declares; a code
no warning has is an error (Z0401). `note` returns nothing: it is a statement, and its result
cannot be bound or passed on (Z0206). R has `message`.

## Arguments
- message: The text to print, a Str.
- code: A warning's code to print the note under, `W` and four digits, written bare (`code: W0703`) or as text; without it the note carries no code.

## Value
Nothing.

## Details
Exact: the text is printed as given, after the word `note:` (or `note[CODE]:` with a code), and
the line number follows in parentheses. A note is never refused by `--deny-warnings` unless it
carries a code, and a pragma `// zeff: allow(CODE) reason` silences a coded note as it silences a
warning (section 11.1).

## A common mistake
Giving a code that no warning has. A note's code is one of the engine's own:

> note("three rows were re-tests", code: W9999)
error[Z0401]: no warning has the code `W9999`
  = help: copy the code from the warning itself; `zeff help W0203` explains a code

Leave `code:` out for a note of your own, `note("three rows were re-tests")`.

## Coming from R or Stata
| note("text") | message("text") | display as text "text" |

R's `message` writes to the error stream, apart from the output; here a note is printed under the
output of its statement, with the line, and `zeff run --quiet` hides it.

## Related tasks
- To flag something that wants attention, use `warn`.
- To print a value as output, use `print`.

## References
Language Reference, section 5.4 ("Errors, warnings and failed draws") and section 10.2 ("What
Zeff prints").

## See also
warn, error, print

## Examples
// A note of the script's own: the text, and the line it came from.
> note("one row in ten is a re-test and was kept")
note: one row in ten is a re-test and was kept (line 1)

// Under one of the engine's codes.
> note("three rows were re-tests", code: W0703)
note[W0703]: three rows were re-tests (line 2)

// The script carries on.
> print("the run carries on")
the run carries on
