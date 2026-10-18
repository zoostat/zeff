# cos

## Summary
Cosine, x in radians

## Description
The cos of an angle given in radians, as a Float; degrees are converted with `x * PI / 180`. An
infinite angle has no cos and gives `NA` with a warning (W0205); `NA` gives `NA`. Given a column,
`cos` applies row by row and returns a column of Floats.

## Arguments
- x: The angle in radians, an Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target, after an exact reduction of
the angle, so that large angles are as accurate as small ones. A multiple of pi is not exactly
representable, so `cos(PI)` is not exactly what the mathematics says but what the Float nearest to
pi gives.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
sin, tan, acos

## Examples
> cos(0)
1
> cos(PI)
-1
> cos(1)
0.540302
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> cos(wages["hours"])
Column<Float> · 3 values
-0.666938
-0.903692
NA
