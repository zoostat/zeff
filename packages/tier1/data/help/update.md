# update

## Summary
Change the values of a variable, everywhere or where a condition holds

## Description
A frame with the values of existing variables changed as the definitions say, in every row or,
with `where:`, only in the rows where the condition is true: `update(hours = 38, where:
is_missing(hours))` fills the gaps and leaves the other rows as they were. A variable keeps its type
and its metadata, so new values must be of that type (an Int may go into a Float variable); to
change a variable's type, `add` replaces it. Rows where the condition is `NA` are left unchanged
and counted in a note. A name the frame does not have is an error that points to `add`. Like
dplyr's `mutate` with `if_else` and Stata's `replace ... if`.

## Arguments
- data: The frame.
- where: The rows to change: an expression over the frame's variables that is true or false in
  every row. Every row when omitted.
- ...: Definitions `name = expr`, each naming a variable of the frame.

## Value
A `Frame` with the same rows and variables, the values changed where asked.

## See also
add, where, is_missing

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.update(hours = 38, where: is_missing(hours))
frame · 3 rows × 3 variables

      id   wage   hours
  1    1   23.4      40
  2    2     19      35
  3    3   31.2      38
> wages.update(wage = wage * 2, where: hours > 36)
frame · 3 rows × 3 variables

      id   wage   hours
  1    1   46.8      40
  2    2     19      35
  3    3   31.2      NA
note: 1 row where `hours > 36` was NA was left unchanged (line 3)
