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

## Missing values
`NA` in either coordinate gives `NA`. In columns each row is taken by itself, and nothing is printed.

## Details
Within two units in the last place of the exact value on every target, with the IEEE values on the
axes and at the origin.

## Coming from R or Stata
| atan2(y, x) | atan2(y, x) | atan2(y, x) |

The three take the vertical coordinate first. A spreadsheet takes the horizontal one first, which
is the usual surprise for a reader coming from one.

## Related tasks
- For the angle of a slope alone, use `atan`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
atan, sin, cos

## Examples
// The first quadrant: an eighth of a turn.
> atan2(1, 1)
0.785398

// The second quadrant, which `atan(y / x)` cannot tell from the fourth.
> atan2(1, -1)
2.35619

// On the negative x axis: a half turn.
> atan2(0, -1)
3.14159

// Row by row, one point in each quadrant, in degrees.
> frame(y = [1, 1, -1, -1], x = [1, -1, -1, 1]).add(angle = atan2(y, x) * 180 / PI)
frame · 4 rows × 3 variables

       y    x   angle
  1    1    1      45
  2    1   -1     135
  3   -1   -1    -135
  4   -1    1     -45
