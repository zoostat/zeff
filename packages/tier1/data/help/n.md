# n

## Summary
Row count, as a reducer inside add and aggregate

## Description
The number of rows of the frame a definition is being evaluated for: inside `add` or `update`, the
whole frame, or the group under `by:`. It is the reducer that turns a count into a share,
`add(share = x / n())`, as dplyr's `n()` and Stata's `_N`. Outside a frame method there is no frame
to count, and `n()` is an error that points to `n_rows`.

## Arguments

## Value
An `Int`.

## Details
Exact: the number of rows of the frame, or of the group, that the definition is evaluated for.

## References
Language Reference, section 6 ("Statistics as methods and reducers").

## See also
n_rows, n_distinct, add

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.add(share = wage / sum(wage), rows = n())
frame · 3 rows × 5 variables

      id   wage   hours      share   rows
  1    1   23.4      40   0.317935      3
  2    2     19      35   0.258152      3
  3    3   31.2      NA   0.423913      3
