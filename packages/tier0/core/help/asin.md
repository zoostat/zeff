# asin

## Summary
Inverse sine

## Description
The angle in radians, between -pi/2 and pi/2, whose sine is `x`, as a Float. A sine lies between -1
and 1, so `x` outside that range has no inverse sine and gives `NA` with a warning (W0205); `NA`
gives `NA`. Given a column, `asin` applies row by row and returns a column of Floats.

## Arguments
- x: A value between -1 and 1, an Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
sin, acos, atan

## Examples
> asin(1)
1.5708
> asin(0.5)
0.523599
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> asin(wages["wage"] / 100)
Column<Float> · 3 values
0.23619
0.191162
0.317297
