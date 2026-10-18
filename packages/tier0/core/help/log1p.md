# log1p

## Summary
log(1 + x), accurate near zero

## Description
The natural logarithm of `1 + x`, computed accurately when `x` is small, where `log(1 + x)` would
lose its leading digits to rounding: `log1p(1e-10)` is `1e-10` to full precision. It is the
transformation for a variable with zeros, such as income with non-earners: `log1p(0)` is `0` where
`log(0)` is `-Inf`. `x = -1` gives `-Inf` with a warning (W0204) and `x` below `-1` gives `NA` with
a warning (W0205); `NA` gives `NA`. Given a column, `log1p` applies row by row and returns a column
of Floats.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target. `expm1` is its inverse.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
log, expm1, exp

## Examples
> log1p(0)
0
> log1p(1e-10)
1e-10
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> log1p(wages["hours"])
Column<Float> · 3 values
3.71357
3.58352
NA
