# where

## Summary
Keep the rows where a condition is true

## Description
Returns the rows for which `condition` is `true`. Rows where it is `NA` are dropped and counted, so
`where(age > 30)` and `where(!(age > 30))` both drop rows with missing age, and the note of each says
so; the two never partition a frame silently. Stata's `keep if age > 30` keeps missing age.

Inside the condition a bare name is a variable of the frame; a script binding of the same name is
used only when the frame has no such variable, and both at once is a warning (W0102). Row order is
kept, and every variable's metadata and the frame's are carried.

## Arguments
- data: The frame to filter.
- condition: An expression over the frame's variables that is true or false in every row, such as
  `hours > 36 && !is_missing(wage)`. A single Bool, `NA`, or a column of another type is an
  error that names the problem (Z0203).

## Value
A `Frame` with the same variables and a subset of the rows.

## See also
select, drop, is_missing, head

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.where(hours > 36)
frame · 1 row × 3 variables

      id   wage   hours
  1    1   23.4      40
note: 1 row dropped where `hours > 36` was NA; `wages.where(is_missing(hours))` lists them (line 2)
> wages.where(is_missing(hours))
frame · 1 row × 3 variables

      id   wage   hours
  1    3   31.2      NA
> wages.where(wage > 100).n_rows()
0
note: no row kept: the result is empty (line 4)
