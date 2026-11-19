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

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Within two units in the last place of the exact value on every target. `log1p` is its inverse.

## Coming from R or Stata
| expm1(x) | expm1(x) | expm1(x) |

Stata has the function in its recent releases only; before them `exp(x) - 1` was the only form
there, and it loses its digits near zero.

## Related tasks
- To undo it, use `log1p`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
exp, log1p

## Examples
// Nothing at zero.
> expm1(0)
0

// Accurate where `exp(x) - 1` loses its digits.
> expm1(1e-10)
1e-10

// A coefficient of 0.05 on a log outcome is a change of 5.13 per cent.
> expm1(0.05)
0.0512711

// Row by row on the shipped wages: `log1p` undone, so `back` equals `exper`.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(lexper = log1p(exper), back = expm1(lexper))
frame · 5 rows × 7 variables · panel worker × year

      worker   year    wage   hours   exper     lexper   back
  1       19   2012   12.64    29.2       0          0      0
  2       19   2013   25.18    31.4       0          0      0
  3       19   2014   35.49    33.5       1   0.693147      1
  4       19   2015   26.07    58.2       2    1.09861      2
  5       19   2016      NA      30       3    1.38629      3
