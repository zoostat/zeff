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

## Missing values
It counts them. In a list, an `NA` item is counted; a `Failed` item is neither missing nor present and is skipped with warning W0801.

## A common mistake
Giving a list of variables inside `add`, to count the missing values across each row. The list has two items, both variables and neither `NA`, so the count would be 0 in every row; the call is refused, and the help says how to count across a row:

> frame(x1 = [1, NA], x2 = [NA, NA]).add(k = n_missing([x1, x2]))
error[Z0219]: `x2` holds only `NA`, so its type is unknown
  = help: give the column at least one value beside the `NA`s

Add the tests up: `add(k = to(is_missing(x1), Int) + to(is_missing(x2), Int))`.

## Coming from R or Stata
| n_missing(x) | sum(is.na(x)) | count if missing(x) |

For every variable of a frame at once, `describe` shows the count, as `misstable summarize` does
in Stata and `colSums(is.na(d))` in R.

## Related tasks
- To test each value, use `is_missing`.
- For the count of every variable of a frame, use `describe`.

## See also
is_missing, fill_missing, len, coalesce

## Examples
// Two of four items are missing.
> n_missing([1, NA, 3, NA])
2

// None.
> n_missing([1, 2, 3])
0

// A column of the shipped wages: 211 rows have no wage.
> n_missing(dataset(wages)["wage"])
211

// Across variables in each row, as a sum of tests.
> let e = frame(x1 = [1, NA, 3], x2 = [NA, NA, 1])
> e.add(k = to(is_missing(x1), Int) + to(is_missing(x2), Int))
frame · 3 rows × 3 variables

      x1   x2   k
  1    1   NA   1
  2   NA   NA   2
  3    3    1   0
