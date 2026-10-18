# floor

## Summary
Largest whole number not above x

## Description
The largest whole number that is not above `x`, as a Float: `floor(2.7)` is `2` and `floor(-2.7)` is
`-3`. An Int is returned unchanged, as a Float, and `NA` gives `NA`. Given a column, `floor` applies
row by row and returns a column of Floats.

The result is a Float, as in R, so that `floor(x)` on a Float column keeps the column's type; `to(x,
Int)` makes an Int of it (release 0.5.0).

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Exact: a Float is already a whole number when its magnitude is 2^52 or more, and below that the
whole number is representable. `Inf` and `-Inf` are returned as they are.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.9 (roundToIntegral).

## See also
ceil, truncate, round

## Examples
> floor(2.7)
2
> floor(-2.7)
-3
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> floor(wages["wage"])
Column<Float> · 3 values
23
19
31
