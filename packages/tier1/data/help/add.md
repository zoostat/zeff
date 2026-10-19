# add

## Summary
Add variables computed from the frame

## Description
A frame with new variables, each defined as `name = expr`, where `expr` is an expression over the
frame's variables (`pay = wage * hours`) or a single value written into every row (`year = 2026`).
Definitions are evaluated left to right, and a later one may use an earlier one in the same call:
`add(lwage = log(wage), high = lwage > 3)`. A definition whose name the frame already has replaces
that variable, and a note says so; `update` changes an existing variable's values without changing
its type. Unchanged columns are shared with the parent, not copied, and every variable keeps its
metadata. Like dplyr's `mutate` and Stata's `generate`.

Reducers inside a definition, such as `dev = wage - mean(wage)`, and `by:` arrive later in this
release.

## Arguments
- data: The frame.
- ...: Definitions `name = expr`: an expression over the frame's variables, or a Bool, Int, Float
  or Str value for every row.

## Value
A `Frame` with the same rows and the variables added or replaced.

## See also
update, select, rename

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.add(pay = wage * hours, double = pay * 2)
frame · 3 rows × 5 variables

      id   wage   hours   pay   double
  1    1   23.4      40   936     1872
  2    2     19      35   665     1330
  3    3   31.2      NA    NA       NA
> wages.add(wage = wage / 10, year = 2026)
frame · 3 rows × 4 variables

      id   wage   hours   year
  1    1   2.34      40   2026
  2    2    1.9      35   2026
  3    3   3.12      NA   2026
note: `wage` replaced the variable of that name (line 3)
