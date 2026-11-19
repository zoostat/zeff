# floor

## Summary
Largest whole number not above x

## Description
The largest whole number that is not above `x`, as a Float: `floor(2.7)` is `2` and `floor(-2.7)` is
`-3`. An Int is returned unchanged, as a Float, and `NA` gives `NA`. Given a column, `floor` applies
row by row and returns a column of Floats.

The result is a Float, as in R, so that `floor(x)` on a Float column keeps the column's type; `to(x,
Int)` makes an Int of it.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact: a Float is already a whole number when its magnitude is 2^52 or more, and below that the
whole number is representable. `Inf` and `-Inf` are returned as they are.

## Coming from R or Stata
| floor(x) | floor(x) | floor(x) |

The three agree, and each returns a number with a fraction of zero and not an integer type.

## Related tasks
- For a whole-number quotient, use `div`.
- To round to the nearest whole number, use `round`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.9 (roundToIntegral).

## See also
ceil, truncate, round

## Examples
// Down to the whole number below.
> floor(2.7)
2

// For a negative number, down is away from zero.
> floor(-2.7)
-3

// Row by row on the shipped wages: each wage in whole units.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(whole = floor(wage))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   whole
  1       19   2012   12.64    29.2       0      12
  2       19   2013   25.18    31.4       0      25
  3       19   2014   35.49    33.5       1      35
  4       19   2015   26.07    58.2       2      26
  5       19   2016      NA      30       3      NA
