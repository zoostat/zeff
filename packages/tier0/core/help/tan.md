# tan

## Summary
Tangent, x in radians

## Description
The tan of an angle given in radians, as a Float; degrees are converted with `x * PI / 180`. An
infinite angle has no tan and gives `NA` with a warning (W0205); `NA` gives `NA`. Given a column,
`tan` applies row by row and returns a column of Floats.

## Arguments
- x: The angle in radians, an Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. An infinite angle also gives `NA`, with warning W0205; in a column the rows that did so are counted in one warning, and no row is dropped.

## Details
Within two units in the last place of the exact value on every target, after an exact reduction of
the angle, so that large angles are as accurate as small ones. A multiple of pi is not exactly
representable, so `tan(PI)` is not exactly what the mathematics says but what the Float nearest to
pi gives.

## A common mistake
Giving the angle in degrees is the likeliest mistake, and it prints nothing: `tan(90)` is the tangent of 90 radians. Convert first, `tan(90 * PI / 180)`. What `tan` itself warns of is an angle that is infinite, as after a division by zero:

> tan(Inf)
NA
warning[W0205]: tan of an infinite value gave NA (line 5)
  = help: an angle must be finite; `where(is_finite(x))` keeps the rows that are

Test the angle first with `is_finite`, or keep the rows that have one with `where(is_finite(x))`.

## Coming from R or Stata
| tan(x) | tan(x) | tan(x) |
| tan(x * PI / 180) | tan(x * pi / 180) | tan(x * _pi / 180) |

All three take radians, and none has a form for degrees. For an infinite angle R returns `NaN`
with a warning and Stata missing without a word; Zeff returns `NA` and counts the rows.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
sin, cos, atan

## Examples
// No angle, no slope.
> tan(0)
0

// An eighth of a turn: a slope of one.
> tan(PI / 4)
1

// Thirty degrees, converted to radians first.
> tan(30 * PI / 180)
0.57735

// Row by row, on angles given in degrees; the missing angle stays missing.
> frame(degrees = [0, 30, 90, 180, NA]).add(tangent = tan(degrees * PI / 180))
frame · 5 rows × 2 variables

      degrees        tangent
  1         0              0
  2        30        0.57735
  3        90    1.63312e+16
  4       180   -1.22465e-16
  5        NA             NA
