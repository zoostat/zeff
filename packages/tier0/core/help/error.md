# error

## Summary
End the script with an error

## In brief
- `error("the sample has no treated units")`: stops the script there, with that message, under
  the code Z0001.
- `error("...", code: Z0401, argument: "x")`: in the file of a built-in written in Zeff only, an
  error under one of the engine's codes, marked on one argument.

## Description
Stops the script at once with `message` as the error's text, under the code Z0001: the way for a
script to refuse to continue when the data or the run is not what its author requires, such as a
sample with no treated units. The statements before it have run and their output stands; nothing
after it runs; the exit code is 1. There is no `try`/`catch` in Zeff: an analysis that fails at the
top level stops where it fails (section 5.4 of the Language Reference). R has `stop`; Stata has
`error`.

## Arguments
- message: The text of the error, a Str.
- code: For the engine's own functions: in the file of a built-in written in Zeff, one of the
  engine's error codes, `code: Z0401`, under which the error is raised. A script's error is
  always Z0001, and `code:` written in a script is refused (Z0404).
- argument: For the engine's own functions: the name of the parameter at fault, as text, so
  that the error is marked on that argument of the call. Refused in a script, as `code:` is.

## Value
Nothing: the script ends.

## Details
Exact: the text is printed as given, after `error[Z0001]:`, with the line and the call marked.

## A common mistake
Giving `code:` or `argument:` in a script. They belong to the engine's own functions; a script's error is always Z0001:

> error("the sample has no treated units", code: Z0401, argument: "x")
error[Z0404]: `code:` of `error` is for the engine's own functions; a script's error is Z0001
  = help: write `error(message)`; `warn(message, code: W0205)` gives a warning one of the engine's codes

Write the message alone: `error("the sample has no treated units")`.

## Coming from R or Stata
| error("text") | stop("text") | display as error "text", then exit 198 |

Stata's `error` takes a number and prints the message that number has; a message of the
do-file's own is printed first and the do-file is left with `exit`. R's `stop` can be caught with
`tryCatch`; here an error is kept from ending the script only by `attempt`, around the one
expression that may fail.

## Related tasks
- To stop only when a condition fails, use `assert`.
- To flag something and carry on, use `warn`.
- To carry on after an expression that may fail, use `attempt`.

## References
Language Reference, sections 5.4 and 5.6 (Z0001).

## See also
warn, print

## Examples
// The statements before it run, and their output stands.
> print("the checks that come before it run")
the checks that come before it run

// The script stops here with the message, and nothing after it runs.
> error("the sample has no treated units")
error[Z0001]: the sample has no treated units
 --> examples.zeff:2:1
  |
2 | error("the sample has no treated units")
  | ^^^^^ the script stops here
