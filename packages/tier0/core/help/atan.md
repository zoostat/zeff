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

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Within two units in the last place of the exact value on every target.

## Coming from R or Stata
| atan(x) | atan(x) | atan(x) |
| atan(x) * 180 / PI | atan(x) * 180 / pi | atan(x) * 180 / _pi |

The three agree: the result is in radians, between a quarter turn below and a quarter turn above.

## Related tasks
- For the angle of a point, with its quadrant, use `atan2`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
atan2, tan, asin

## Examples
// A slope of one is an eighth of a turn.
> atan(1)
0.785398

// In degrees.
> atan(1) * 180 / PI
45

// Row by row, the angle of each slope in degrees.
> frame(slope = [-1, 0, 0.5, 1, NA]).add(angle = atan(slope) * 180 / PI)
frame · 5 rows × 2 variables

      slope     angle
  1      -1       -45
  2       0         0
  3     0.5   26.5651
  4       1        45
  5      NA        NA
