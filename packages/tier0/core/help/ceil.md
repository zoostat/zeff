# ceil

## Summary
Smallest whole number not below x

## Description
The smallest whole number that is not below `x`, as a Float: `ceil(2.1)` is `3` and `ceil(-2.7)` is
`-2`. An Int is returned unchanged, as a Float, and `NA` gives `NA`. Given a column, `ceil` applies
row by row and returns a column of Floats. R calls this function `ceiling`.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Exact, as `floor` is. `Inf` and `-Inf` are returned as they are.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.9 (roundToIntegral).

## See also
floor, truncate, round

## Examples
> ceil(2.1)
3
> ceil(-2.7)
-2
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> ceil(wages["wage"])
Column<Float> · 3 values
24
19
32
