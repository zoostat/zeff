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

## Missing values
`NA` gives `NA`. Zero gives `-Inf` with warning W0204 and a negative number gives `NA` with warning W0205; in a column the rows of each are counted in one warning.

## Details
Within two units in the last place of the exact value on every target; exact at the powers of two.

## A common mistake
Taking the logarithm of zero. It is `-Inf`, and the warning says so:

> log2(0)
-Inf
warning[W0204]: log2 of zero gave -Inf (line 5)
  = help: `log1p(x)` is log(1 + x) and is finite at zero; or keep the rows with `where(x > 0)`

Keep the positive rows with `where(x > 0)` before taking the logarithm.

## Coming from R or Stata
| log2(x) | log2(x) | ln(x) / ln(2) |

Stata has no function for base 2. R gives `-Inf` for zero without a word and `NaN` for a negative
number with a warning; Zeff warns for both.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
log, log10, exp

## Examples
// A power of two is exact.
> log2(8)
3

// A half.
> log2(0.5)
-1

// Row by row on the shipped wages: how many doublings each year's hours are.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(doublings = log2(hours))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   doublings
  1       19   2012   12.64    29.2       0      4.8679
  2       19   2013   25.18    31.4       0     4.97269
  3       19   2014   35.49    33.5       1     5.06609
  4       19   2015   26.07    58.2       2     5.86295
  5       19   2016      NA      30       3     4.90689
