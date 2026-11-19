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

## Missing values
`NA` gives `NA`. `-1` gives `-Inf` with warning W0204 and a number below `-1` gives `NA` with warning W0205; in a column the rows of each are counted in one warning.

## Details
Within two units in the last place of the exact value on every target. `expm1` is its inverse.

## A common mistake
Giving a number below -1, where `1 + x` is negative and has no logarithm. The result is `NA`, and the warning says so:

> log1p(-2)
NA
warning[W0205]: log1p below -1 gave NA (line 5)
  = help: log1p(x) is log(1 + x), which has no real value below -1; `where(x > -1)` keeps the rows that have one

`log1p` is for numbers from -1 upwards: counts, incomes, years.

## Coming from R or Stata
| log1p(x) | log1p(x) | ln1p(x) or log1p(x) |

Stata has the function in its recent releases only, under both names. Below -1 R returns `NaN`
with a warning and Stata missing without a word; Zeff returns `NA` and counts the rows.

## Related tasks
- To undo it, use `expm1`.
- For a variable that is never zero, `log` is the usual choice.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
log, expm1, exp

## Examples
// Finite at zero, where `log` is not.
> log1p(0)
0

// Accurate where `log(1 + x)` loses its digits.
> log1p(1e-10)
1e-10

// Row by row on the shipped wages: years of experience, which begin at zero.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(lexper = log1p(exper))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper     lexper
  1       19   2012   12.64    29.2       0          0
  2       19   2013   25.18    31.4       0          0
  3       19   2014   35.49    33.5       1   0.693147
  4       19   2015   26.07    58.2       2    1.09861
  5       19   2016      NA      30       3    1.38629
