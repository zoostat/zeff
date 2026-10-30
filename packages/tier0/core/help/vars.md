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
second, and each fit keeps its variable's name. Like Stata's global macros holding a varlist, with
the names checked rather than pasted.

## Arguments
- name: The first variable, a bare name, or a range of variables, `x1..x9`.
- ...: Further variables or ranges.

## Value
A `Vars`, the list of names in the order written, which prints as the names separated by commas.

## See also
variables, select, is

## Examples
> vars(wage, hours)
wage, hours
> let w = dataset(wages)
> for y in vars(wage, hours) {
>     print(w.mean(y))
> }
27.3695
37.9446
warning: 211 rows with NA in wage were skipped (line 3)
warning: 35 rows with NA in hours were skipped (line 3)
