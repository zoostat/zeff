# name_of

## Summary
The name of a variable as text

## Description
Gives the name a variable is written with, as a Str: `name_of(educ)` is `"educ"`. A `Var` has no
text form of its own, so a function that takes variables and wants to say which it used, as
`drop_missing` names the variables it looked at in its note, writes their names into its message
with `name_of` (and `join`, where it puts several together). Given `NA` (an optional `Var` parameter left out) it gives `NA`. R has
`deparse(substitute(x))`; Stata has the macro the variable was passed in.

## Arguments
- v: The variable, written bare (`educ`) or held in a `Var` parameter or binding.

## Value
The name, a Str.

## Missing values
`name_of` reads the name and none of the variable's values, so missing values in the variable play no part. Given `NA` in place of a variable, an optional parameter left out, it gives `NA`.

## Details
Exact: the name as written, without quotes or the frame it belongs to.

## Coming from R or Stata
| name_of(v) | deparse(substitute(v)) | "`v'" |

In R the name must be taken before the argument is evaluated, which is easy to get wrong inside
nested functions; in Stata a variable is passed as a name already. Here a `Var` is a name with a
type of its own, and `name_of` is the one way to its text.

## Related tasks
- To name several variables outside a frame, use `vars`.
- For the names of every variable of a frame, use `variables`.

## See also
warn, note, vars, print

## Examples
// The name as text.
> name_of(educ)
educ

// In a loop over variables of the shipped wages, to say which is which.
> let w = dataset(wages)
> for v in vars(wage, hours) {
>     print(name_of(v), w.mean(v))
> }
wage 27.3695
hours 37.9446
warning[W0201]: 211 rows with NA in wage were skipped (line 3)
warning[W0201]: 35 rows with NA in hours were skipped (line 3)
