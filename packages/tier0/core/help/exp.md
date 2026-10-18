# exp

## Summary
e raised to x

## Description
The exponential function, e to the power `x`, as a Float: `exp(0)` is `1` and `exp(1)` is `E`. A
large `x` overflows to `Inf` and a very negative one underflows to `0`, as in R, without a warning;
`NA` gives `NA`. Given a column, `exp` applies row by row and returns a column of Floats. It undoes
`log`: `exp(log(x))` is `x`, so a coefficient on a log-wage equation turns back into a wage ratio
with `exp`.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target, from one implementation
compiled into the engine (Engine and Project Specification, section 8.3, point 3). `expm1` is the
accurate form of `exp(x) - 1` near zero.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
log, expm1, E

## Examples
> exp(0)
1
> exp(1)
2.71828
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> exp(wages["hours"] / 10)
Column<Float> · 3 values
54.5982
33.1155
NA
