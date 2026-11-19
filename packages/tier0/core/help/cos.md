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

## Missing values
`NA` gives `NA`. An infinite angle also gives `NA`, with warning W0205; in a column the rows that did so are counted in one warning, and no row is dropped.

## Details
Within two units in the last place of the exact value on every target, after an exact reduction of
the angle, so that large angles are as accurate as small ones. A multiple of pi is not exactly
representable, so `cos(PI)` is not exactly what the mathematics says but what the Float nearest to
pi gives.

## A common mistake
Giving the angle in degrees is the likeliest mistake, and it prints nothing: `cos(90)` is the cosine of 90 radians. Convert first, `cos(90 * PI / 180)`. What `cos` itself warns of is an angle that is infinite, as after a division by zero:

> cos(Inf)
NA
warning[W0205]: cos of an infinite value gave NA (line 5)
  = help: an angle must be finite; `where(is_finite(x))` keeps the rows that are

Test the angle first with `is_finite`, or keep the rows that have one with `where(is_finite(x))`.

## Coming from R or Stata
| cos(x) | cos(x) | cos(x) |
| cos(x * PI / 180) | cos(x * pi / 180) | cos(x * _pi / 180) |

All three take radians, and none has a form for degrees. For an infinite angle R returns `NaN`
with a warning and Stata missing without a word; Zeff returns `NA` and counts the rows.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
sin, tan, acos

## Examples
// No angle.
> cos(0)
1

// A half turn.
> cos(PI)
-1

// Sixty degrees, converted to radians first.
> cos(60 * PI / 180)
0.5

// Row by row, on angles given in degrees; the missing angle stays missing.
> frame(degrees = [0, 30, 90, 180, NA]).add(cosine = cos(degrees * PI / 180))
frame · 5 rows × 2 variables

      degrees        cosine
  1         0             1
  2        30      0.866025
  3        90   6.12323e-17
  4       180            -1
  5        NA            NA
