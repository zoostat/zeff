# name_of

## Summary
The name of a variable as text

## Description
Gives the name a variable is written with, as a Str: `name_of(educ)` is `"educ"`. A `Var` has no
text form of its own, so a function that takes variables and wants to say which it used, as
`drop_missing` names the variables it looked at in its note, writes their names into its message
with `name_of` (and `join`, once the text package ships later in release 0.7.0). Given `NA` (an optional `Var` parameter left out) it gives `NA`. R has
`deparse(substitute(x))`; Stata has the macro the variable was passed in.

## Arguments
- v: The variable, written bare (`educ`) or held in a `Var` parameter or binding.

## Value
The name, a Str.

## Details
Exact: the name as written, without quotes or the frame it belongs to.

## See also
warn, note, vars, print

## Examples
> name_of(educ)
educ
> print(name_of(educ))
educ
