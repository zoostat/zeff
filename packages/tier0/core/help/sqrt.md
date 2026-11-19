# sqrt

## Summary
Square root

## Description
The square root of a number, as a Float. A negative number has no real square root: the result is
`NA` with a warning (W0205), as section 4.1 of the Language Reference has it, never a complex number
or a silent `NaN`. `NA` gives `NA`. Given a column, `sqrt` applies row by row, returns a column, and
counts the rows that gave `NA` in one warning.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. A negative number also gives `NA`, with warning W0205; in a column the rows that did so are counted in one warning, and no row is dropped.

## Details
Correctly rounded: the square root is the one operation beyond the four of arithmetic that IEEE 754
requires to be exact to the last bit, so every target gives the same result. `sqrt(0)` is `0` and
`sqrt(Inf)` is `Inf`.

## A common mistake
Taking the root of a number that can be negative, such as a difference. There is no real square root, so the result is `NA`, and the warning says so:

> sqrt(-4)
NA
warning[W0205]: sqrt of a negative number gave NA (line 5)
  = help: a negative number has no real square root; `where(x >= 0)` keeps the rows that have one

Take the root of the size of the number, `sqrt(abs(-4))`, or keep the rows that have a root with `where(x >= 0)`.

## Coming from R or Stata
| sqrt(x) | sqrt(x) | sqrt(x) |

For a negative number R returns `NaN` with a warning that names no row count, and Stata returns
missing without a word. Zeff returns `NA` and says how many rows did so.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.4.1 (squareRoot).

## See also
abs, exp, log

## Examples
// A perfect square.
> sqrt(16)
4

// Six digits are printed; the value is correct to the last bit.
> sqrt(2)
1.41421

// Row by row on the shipped wages: the root of each year's hours.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(root = sqrt(hours))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper      root
  1       19   2012   12.64    29.2       0    5.4037
  2       19   2013   25.18    31.4       0   5.60357
  3       19   2014   35.49    33.5       1   5.78792
  4       19   2015   26.07    58.2       2   7.62889
  5       19   2016      NA      30       3   5.47723
