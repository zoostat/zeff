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

## Missing values
`NA` gives `NA`. Zero gives `-Inf` with warning W0204 and a negative number gives `NA` with warning W0205; in a column the rows of each are counted in one warning, and no row is dropped.

## Details
Within two units in the last place of the exact value on every target, from one implementation
compiled into the engine. A base of 2 or 10 uses the dedicated `log2` and `log10`; any other base
is `log(x) / log(base)`. A base that is not positive, or is 1, gives `NA` with a warning. A base
below 1 makes the logarithm of zero `Inf` rather than `-Inf`.

## A common mistake
Taking the logarithm of a variable that has zeros, such as years of experience. The logarithm of zero is `-Inf`, which a later mean or regression cannot use, and the warning names the way out:

> log(0)
-Inf
warning[W0204]: log of zero gave -Inf (line 6)
  = help: `log1p(x)` is log(1 + x) and is finite at zero; or keep the rows with `where(x > 0)`

Use `log1p(x)`, which is `log(1 + x)` and is `0` at zero, or keep the positive rows with `where(x > 0)`.

## Coming from R or Stata
| log(x) | log(x) | ln(x) or log(x) |
| log(x, base: 2) | log(x, base = 2) or log2(x) | ln(x) / ln(2) |
| log(x, base: 10) | log10(x) | log10(x) |

In all three `log` is the natural logarithm. R gives `-Inf` for zero without a word and `NaN` for a
negative number with a warning; Stata turns both into missing without a word, so the rows vanish
from a regression unnoticed. Zeff warns for both and counts the rows.

## Related tasks
- For a variable with zeros, use `log1p`.
- To undo a logarithm, use `exp`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
exp, log2, log10, log1p

## Examples
// The natural logarithm.
> log(1)
0

// Another base.
> log(8, base: 2)
3

// Row by row on the shipped wages: the log wage of one worker; the missing wage stays missing.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(lwage = log(wage))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper     lwage
  1       19   2012   12.64    29.2       0   2.53687
  2       19   2013   25.18    31.4       0   3.22605
  3       19   2014   35.49    33.5       1   3.56925
  4       19   2015   26.07    58.2       2   3.26079
  5       19   2016      NA      30       3        NA

// A variable with zeros: two rows give `-Inf`, and one warning counts them.
> w.add(lexper = log(exper))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper     lexper
  1       19   2012   12.64    29.2       0       -Inf
  2       19   2013   25.18    31.4       0       -Inf
  3       19   2014   35.49    33.5       1          0
  4       19   2015   26.07    58.2       2   0.693147
  5       19   2016      NA      30       3    1.09861
warning[W0204]: log of zero gave -Inf in 2 rows (line 5)
  = help: `log1p(x)` is log(1 + x) and is finite at zero; or keep the rows with `where(x > 0)`
