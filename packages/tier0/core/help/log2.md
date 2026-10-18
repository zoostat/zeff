# log2

## Summary
Logarithm base 2

## Description
The logarithm of `x` to base 2, as a Float: `log2(8)` is `3`. Zero gives `-Inf` with a warning
(W0204) and a negative number gives `NA` with a warning (W0205), as `log` does; `NA` gives `NA`.
Given a column, `log2` applies row by row and returns a column of Floats.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target; exact at the powers of two.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
log, log10, exp

## Examples
> log2(8)
3
> log2(0.5)
-1
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> log2(wages["hours"])
Column<Float> · 3 values
5.32193
5.12928
NA
