# log10

## Summary
Logarithm base 10

## Description
The logarithm of `x` to base 10, as a Float: `log10(1000)` is `3`. Zero gives `-Inf` with a warning
(W0204) and a negative number gives `NA` with a warning (W0205), as `log` does; `NA` gives `NA`.
Given a column, `log10` applies row by row and returns a column of Floats.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target; exact at the powers of ten.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
log, log2, exp

## Examples
> log10(1000)
3
> log10(0.01)
-2
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> log10(wages["wage"])
Column<Float> · 3 values
1.36922
1.27875
1.49415
