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

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Within two units in the last place of the exact value on every target, from one implementation
compiled into the engine (Engine and Project Specification, section 8.3, point 3). `expm1` is the
accurate form of `exp(x) - 1` near zero.

## Coming from R or Stata
| exp(x) | exp(x) | exp(x) |

A value too large for a Float is `Inf` in Zeff and in R; Stata returns missing. None of the three
warns.

## Related tasks
- For `exp(x) - 1` when `x` is small, use `expm1`.
- For the number e itself, use `E`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
log, expm1, E

## Examples
// e to the power 0.
> exp(0)
1

// e itself.
> exp(1)
2.71828

// A coefficient of 0.08 on a log wage is a wage ratio of about 1.083.
> exp(0.08)
1.08329

// Row by row on the shipped wages: the logarithm undone, so `back` equals `wage`.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(lwage = log(wage), back = exp(lwage))
frame · 5 rows × 7 variables · panel worker × year

      worker   year    wage   hours   exper     lwage    back
  1       19   2012   12.64    29.2       0   2.53687   12.64
  2       19   2013   25.18    31.4       0   3.22605   25.18
  3       19   2014   35.49    33.5       1   3.56925   35.49
  4       19   2015   26.07    58.2       2   3.26079   26.07
  5       19   2016      NA      30       3        NA      NA
