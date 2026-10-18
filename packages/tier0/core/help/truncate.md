# truncate

## Summary
Drop the fractional part

## Description
The whole number nearest to `x` on the side of zero, as a Float: `truncate(2.7)` is `2` and
`truncate(-2.7)` is `-2`, so it is `floor` for positive numbers and `ceil` for negative ones. An
Int is returned unchanged, as a Float, and `NA` gives `NA`. Given a column, `truncate` applies row
by row and returns a column of Floats. R calls this function `trunc`.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Exact, as `floor` is. `Inf` and `-Inf` are returned as they are.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.9 (roundToIntegral).

## See also
floor, ceil, round

## Examples
> truncate(2.7)
2
> truncate(-2.7)
-2
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> truncate(wages["wage"] / 10)
Column<Float> · 3 values
2
1
3
