# vars

## Summary
Variable list outside a frame method, for loops and functions that build lists

## Description
Names one or more variables where there is no frame to look them up in: in a `for` loop that runs
over variables, and in a function that builds a list of them. Inside a frame method a list of bare
names, `[lwage, hours]`, is already a list of variables, so `vars` is not needed there. A range,
`x1..x9`, may stand among the names; it is kept as written and expanded, by position, against the
frame the list meets, so `vars(age, x1..x50)` names fifty-one variables of whichever frame it is
used with. Each variable is checked against that frame when the statement runs.

A loop variable over `vars(...)` stands for the variable it holds: in `for y in vars(lwage, hours)
{ wages.reg(y ~ educ) }` the formula is `lwage ~ educ` in the first pass and `hours ~ educ` in the
second, and each fit keeps its variable's name. A list of exactly one variable is accepted
wherever a function takes one variable, directly or through an option record, so
`{ se: robust, cluster: vars(firm) }` can be named once and spread into every call; a list of
two or more there is an error (Z0401). Like Stata's global macros holding a varlist, with the
names checked rather than pasted.

## Arguments
- name: The first variable, a bare name, or a range of variables, `x1..x9`.
- ...: Further variables or ranges.

## Value
A `Vars`, the list of names in the order written, which prints as the names separated by commas.

## Missing values
`vars` names variables and reads none of their values, so missing values play no part in it.

## Coming from R or Stata
| vars(wage, hours) | c("wage", "hours") | local xs wage hours |
| vars(x1..x9) | paste0("x", 1:9) | local xs x1-x9 |

In R the names are text, or are picked by the selection helpers of `dplyr`. In Stata a macro is
pasted into the command as it stands, so a misspelt name is found only where the command fails;
here each name is checked against the frame the list meets, and the message names the frame.

## Related tasks
- For the names of a frame's variables, use `variables`.
- To keep some variables of a frame, use `select`.

## See also
variables, select, is

## Examples
// A list of two variables, named with no frame in sight.
> vars(wage, hours)
wage, hours

// A loop over variables of the shipped wages: the loop variable stands for the variable it holds.
> let w = dataset(wages)
> for y in vars(wage, hours) {
>     print(w.mean(y))
> }
27.3695
37.9446
warning[W0201]: 211 rows with NA in wage were skipped (line 3)
warning[W0201]: 35 rows with NA in hours were skipped (line 3)

// The list given to a frame method.
> w.select(vars(worker, year)).n_cols()
2
