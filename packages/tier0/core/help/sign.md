# sign

## Summary
-1, 0 or 1

## Description
The sign of a number as an Int: `-1` for a negative number, `0` for zero and `1` for a positive one;
`NA` gives `NA`. Given a column, `sign` applies row by row and returns a column of Ints.

R's `sign` returns a double; Zeff's returns an Int, since the three values are whole numbers.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
An Int; a column of Ints when `x` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact. Both zeros (`0` and `-0.0`) give `0`; `Inf` gives `1` and `-Inf` gives `-1`.

## Coming from R or Stata
| sign(x) | sign(x) | sign(x) |

The three agree on -1, 0 and 1, and a missing value stays missing. R returns a double; Zeff
returns an Int, so the result can be compared with `==` and used as a count without conversion.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.7.2 (sign of a value).

## See also
abs, is_finite

## Examples
// A negative number.
> sign(-2.5)
-1

// Zero has no sign.
> sign(0)
0

// Row by row on the shipped wages: whether each wage is below or above 25.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(side = sign(wage - 25))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   side
  1       19   2012   12.64    29.2       0     -1
  2       19   2013   25.18    31.4       0      1
  3       19   2014   35.49    33.5       1      1
  4       19   2015   26.07    58.2       2      1
  5       19   2016      NA      30       3     NA
