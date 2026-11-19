# ceil

## Summary
Smallest whole number not below x

## Description
The smallest whole number that is not below `x`, as a Float: `ceil(2.1)` is `3` and `ceil(-2.7)` is
`-2`. An Int is returned unchanged, as a Float, and `NA` gives `NA`. Given a column, `ceil` applies
row by row and returns a column of Floats. R calls this function `ceiling`.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact, as `floor` is. `Inf` and `-Inf` are returned as they are.

## Coming from R or Stata
| ceil(x) | ceiling(x) | ceil(x) |

R spells the name out. The three agree on every value.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.9 (roundToIntegral).

## See also
floor, truncate, round

## Examples
// Up to the whole number above.
> ceil(2.1)
3

// For a negative number, up is towards zero.
> ceil(-2.7)
-2

// Row by row on the shipped wages: each wage raised to a whole unit.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(whole = ceil(wage))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   whole
  1       19   2012   12.64    29.2       0      13
  2       19   2013   25.18    31.4       0      26
  3       19   2014   35.49    33.5       1      36
  4       19   2015   26.07    58.2       2      27
  5       19   2016      NA      30       3      NA
