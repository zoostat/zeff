# abs

## Summary
Absolute value

## Description
The absolute value of a number: `x` itself when it is not negative, `-x` otherwise. An Int stays an Int
and a Float stays a Float, so `abs(-3)` is `3` and `abs(-2.5)` is `2.5`; `NA` gives `NA`. Given a
column, `abs` applies row by row and returns a column.

The one Int that has no absolute value in 64 bits, -9223372036854775808, is an error (Z0211) rather
than a wrong number.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
An Int for an Int, a Float for a Float; a column of the same when `x` is a column.

## Details
Exact: the sign is dropped and nothing is rounded.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.5.1.

## See also
sign, sqrt, max

## Examples
> abs(-3)
3
> abs(-2.5)
2.5
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> abs(wages["wage"] - 25)
Column<Float> · 3 values
1.6
6
6.2
