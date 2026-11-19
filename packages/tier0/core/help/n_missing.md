# n_missing

## Summary
Number of NA values

## Description
How many of the values are `NA`: of a list, or of a column, where it is the count the reducers
report when they skip missing values (W0201). `n_missing(x)` and the count of the rest add up
to `len(x)`. A list that holds failed values from `attempt` counts them neither as missing nor
as present: they are skipped as `len` skips them, with the same warning (W0801). A list of
variables, as `n_missing([x1, x2, x3])` inside `add`, is refused (Z0401): its three items are
variables and none of them is `NA`, so the count would be 0 in every row. The count of missing
values across variables in each row is the sum of the tests,
`to(is_missing(x1), Int) + to(is_missing(x2), Int)`. Like R's `sum(is.na(x))` and the counts of
Stata's `misstable`.

## Arguments
- x: A list or a column.

## Value
An Int, 0 when nothing is missing.

## See also
is_missing, fill_missing, len, coalesce

## Examples
> n_missing([1, NA, 3, NA])
2
> n_missing([1, 2, 3])
0
> let d = frame(x = [1, NA, 3])
> n_missing(d["x"])
1
> let e = frame(x1 = [1, NA, 3], x2 = [NA, NA, 1])
> e.add(k = to(is_missing(x1), Int) + to(is_missing(x2), Int))
frame · 3 rows × 3 variables

      x1   x2   k
  1    1   NA   1
  2   NA   NA   2
  3    3    1   0
