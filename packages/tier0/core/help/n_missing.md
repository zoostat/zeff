# n_missing

## Summary
Number of NA values

## Description
How many of the values are `NA`: of a list, or of a column, where it is the count the reducers
report when they skip missing values (W0201). `n_missing(x)` and the count of the rest add up
to `len(x)`. A list that holds failed values from `attempt` counts them neither as missing nor
as present: they are skipped as `len` skips them, with the same warning (W0801). Like R's
`sum(is.na(x))` and the counts of Stata's `misstable`.

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
