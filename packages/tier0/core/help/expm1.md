# expm1

## Summary
exp(x) - 1, accurate near zero

## Description
`exp(x) - 1` computed accurately when `x` is small, where the subtraction would lose its leading
digits to rounding: `expm1(1e-10)` is `1e-10` to full precision. It turns a small log change back
into a proportional change: a coefficient of 0.05 on a log outcome is a `expm1(0.05)` change of
5.13 per cent. `NA` gives `NA`. Given a column, `expm1` applies row by row and returns a column of
Floats.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target. `log1p` is its inverse.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
exp, log1p

## Examples
> expm1(0)
0
> expm1(1e-10)
1e-10
> expm1(1)
1.71828
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> expm1(wages["wage"] / 100)
Column<Float> · 3 values
0.263644
0.20925
0.366155
