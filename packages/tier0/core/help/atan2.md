# atan2

## Summary
Angle of the point (x, y)

## Description
The angle in radians, between -pi and pi, from the positive x axis to the point `(x, y)`, as a Float:
`atan2(1, 1)` is pi/4 and `atan2(0, -1)` is pi. Unlike `atan(y / x)` it uses the signs of both
coordinates, so every quadrant is told apart and `x = 0` is no division by zero; `atan2(0, 0)` is
`0`. Note the order of the arguments, `y` first, as in R, C and Python. `NA` in either gives `NA`.
Given columns, `atan2` applies row by row and returns a column of Floats.

## Arguments
- y: The vertical coordinate, an Int or a Float, or a column of either.
- x: The horizontal coordinate, likewise.

## Value
A Float; a column of Floats when an argument is a column.

## Details
Within two units in the last place of the exact value on every target, with the IEEE values on the
axes and at the origin.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
atan, sin, cos

## Examples
> atan2(1, 1)
0.785398
> atan2(1, -1)
2.35619
> atan2(0, -1)
3.14159
> atan2(-1, 0)
-1.5708
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 5)
> atan2(wages["wage"], wages["hours"])
Column<Float> · 3 values
0.529317
0.497343
NA
