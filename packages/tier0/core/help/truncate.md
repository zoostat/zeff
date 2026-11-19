# truncate

## Summary
Drop the fractional part

## Description
The whole number nearest to `x` on the side of zero, as a Float: `truncate(2.7)` is `2` and
`truncate(-2.7)` is `-2`, so it is `floor` for positive numbers and `ceil` for negative ones. An
Int is returned unchanged, as a Float, and `NA` gives `NA`. Given a column, `truncate` applies row
by row and returns a column of Floats. R calls this function `trunc`.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact, as `floor` is. `Inf` and `-Inf` are returned as they are.

## Coming from R or Stata
| truncate(x) | trunc(x) | trunc(x) or int(x) |

R and Stata shorten the name. The three agree on every value.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.9 (roundToIntegral).

## See also
floor, ceil, round

## Examples
// The fraction dropped.
> truncate(2.7)
2

// Towards zero for a negative number, where `floor` goes down.
> truncate(-2.7)
-2

// Row by row on the shipped wages: the tens of each wage.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(tens = truncate(wage / 10))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   tens
  1       19   2012   12.64    29.2       0      1
  2       19   2013   25.18    31.4       0      2
  3       19   2014   35.49    33.5       1      3
  4       19   2015   26.07    58.2       2      2
  5       19   2016      NA      30       3     NA
