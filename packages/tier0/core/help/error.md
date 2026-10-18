# error

## Summary
End the script with an error

## Description
Stops the script at once with `message` as the error's text, under the code Z0001: the way for a
script to refuse to continue when the data or the run is not what its author requires, such as a
sample with no treated units. The statements before it have run and their output stands; nothing
after it runs; the exit code is 1. There is no `try`/`catch` in Zeff: an analysis that fails at the
top level stops where it fails (section 5.4 of the Language Reference). R has `stop`; Stata has
`error`.

## Arguments
- message: The text of the error, a Str.

## Value
Nothing: the script ends.

## Details
Exact: the text is printed as given, after `error[Z0001]:`, with the line and the call marked.

## References
Language Reference, sections 5.4 and 5.6 (Z0001).

## See also
warn, print

## Examples
> print("the checks that come before it run")
the checks that come before it run
> error("the sample has no treated units")
error[Z0001]: the sample has no treated units
 --> examples.zeff:2:1
  |
2 | error("the sample has no treated units")
  | ^^^^^ the script stops here
