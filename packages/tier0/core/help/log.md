# log

## Summary
Logarithm

## Description
The natural logarithm of `x`, or the logarithm to `base:` when one is given: `log(8, base: 2)` is
`3`. The logarithm of zero is `-Inf` with a warning (W0204) that points to `log1p` for a variable
with zeros; the logarithm of a negative number has no real value and is `NA` with a warning (W0205),
never a silent `NaN`. `NA` gives `NA`. Given a column, `log` applies row by row, returns a column
of Floats, and counts the rows of each warning once. It is the transformation behind every log-wage
equation.

R's `log(0)` is `-Inf` silently and its `log(-1)` is `NaN` with a warning; Stata's `ln` turns both
into missing silently.

## Arguments
- x: An Int or a Float, or a column of either.
- base: The base, a positive Float other than 1; e by default. `base: 2` and `base: 10` give the
  same values as `log2` and `log10`.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target, from one implementation
compiled into the engine. A base of 2 or 10 uses the dedicated `log2` and `log10`; any other base
is `log(x) / log(base)`. A base that is not positive, or is 1, gives `NA` with a warning. A base
below 1 makes the logarithm of zero `Inf` rather than `-Inf`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
exp, log2, log10, log1p

## Examples
> log(1)
0
> log(8, base: 2)
3
> log(1000, base: 10)
3
> log(0)
-Inf
warning: log of zero gave -Inf (line 4)
  = help: `log1p(x)` is log(1 + x) and is finite at zero; or keep the rows with `where(x > 0)`
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 5)
> log(wages["wage"])
Column<Float> · 3 values
3.15274
2.94444
3.44042
