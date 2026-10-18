# atan

## Summary
Inverse tangent

## Description
The angle in radians, between -pi/2 and pi/2, whose tangent is `x`, as a Float; it is defined for
every number, and `atan(Inf)` is pi/2. `NA` gives `NA`. Given a column, `atan` applies row by row
and returns a column of Floats. For the angle of a point, which needs the signs of both coordinates,
use `atan2`.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
atan2, tan, asin

## Examples
> atan(1)
0.785398
> atan(-1)
-0.785398
> atan(1e300)
1.5708
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> atan(wages["wage"])
Column<Float> · 3 values
1.52809
1.51821
1.53876
