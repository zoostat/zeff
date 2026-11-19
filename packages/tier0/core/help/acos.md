# acos

## Summary
Inverse cosine

## Description
The angle in radians, between 0 and pi, whose cosine is `x`, as a Float. A cosine lies between -1
and 1, so `x` outside that range has no inverse cosine and gives `NA` with a warning (W0205); `NA`
gives `NA`. Given a column, `acos` applies row by row and returns a column of Floats.

## Arguments
- x: A value between -1 and 1, an Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. A value outside -1 to 1 also gives `NA`, with warning W0205; in a column the rows that did so are counted in one warning, and no row is dropped.

## Details
Within two units in the last place of the exact value on every target.

## A common mistake
Giving a value outside -1 to 1, such as a percentage written as 50 where 0.5 is meant. No angle has such a cosine, so the result is `NA`, and the warning says so:

> acos(2)
NA
warning[W0205]: acos outside [-1, 1] gave NA (line 4)
  = help: a sine or cosine lies between -1 and 1, so its inverse has no value elsewhere; `where(abs(x) <= 1)` keeps the rows that have one

Bring the value into the range first: a percentage divided by 100, or a ratio held with `clamp(x, -1, 1)` when rounding has pushed it just past 1.

## Coming from R or Stata
| acos(x) | acos(x) | acos(x) |
| acos(x) * 180 / PI | acos(x) * 180 / pi | acos(x) * 180 / _pi |

The result is in radians in all three. Outside -1 to 1 R returns `NaN` with a warning and Stata
missing without a word; Zeff returns `NA` and counts the rows.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
cos, asin, atan

## Examples
// The angle whose cosine is 1: none.
> acos(1)
0

// In degrees.
> acos(0.5) * 180 / PI
60

// Row by row; the value outside the range gives `NA`, and one warning counts it.
> frame(x = [-1, 0, 0.5, 2, NA]).add(angle = acos(x))
frame · 5 rows × 2 variables

        x     angle
  1    -1   3.14159
  2     0    1.5708
  3   0.5    1.0472
  4     2        NA
  5    NA        NA
warning[W0205]: acos outside [-1, 1] gave NA in 1 row (line 3)
  = help: a sine or cosine lies between -1 and 1, so its inverse has no value elsewhere; `where(abs(x) <= 1)` keeps the rows that have one
