# min

## Summary
Smallest of two or more values, row by row

## Description
The smallest of the values given. With Ints the result is an Int; with any Float it is a Float. Any
`NA` among the values gives `NA`, as section 4.1 of the Language Reference has it and as R's `pmin`
does (Stata's `min` skips missing values instead). Given columns, `min` applies row by row, a scalar
standing for its value in every row, and returns a column: `min(wages["hours"], 38)` caps the hours at
38. The smallest value of one column is the frame method `data.min(var)` of the `data` package.

## Arguments
- a: An Int or a Float, or a column of either.
- b: Another.
- ...: Further values or columns.

## Value
An Int when every value is an Int, a Float otherwise; a column of the same when any argument is a
column.

## Details
Exact. Columns must have the same number of rows (Z0217).

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.6 (minimum and maximum
operations).

## See also
max, abs

## Examples
> min(3, 7.5)
3
> min(4, 2, 9)
2
> min(1, NA)
NA
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> min(wages["hours"], 38)
Column<Int> · 3 values
38
35
NA
