# min

## Summary
Smallest of two or more values, row by row

## Description
The smallest of the values given. With Ints the result is an Int; with any Float it is a Float. Any
`NA` among the values gives `NA`, as section 4.1 of the Language Reference has it and as R's `pmin`
does (Stata's `min` skips missing values instead). Given columns, `min` applies row by row, a scalar
standing for its value in every row, and returns a column: `min(wages["hours"], 38)` caps the hours at
38. The smallest value of one column is the frame method `data.min(var)` of the `data` package.

## Arguments
- a: An Int or a Float, or a column of either.
- b: Another.
- ...: Further values or columns.

## Value
An Int when every value is an Int, a Float otherwise; a column of the same when any argument is a
column.

## Missing values
Any `NA` among the values gives `NA`: the smallest of a known and an unknown number is unknown. In columns each row is taken by itself, and nothing is printed.

## Details
Exact. Columns must have the same number of rows (Z0217).

## Coming from R or Stata
| min(a, b) | pmin(a, b) | min(a, b) |
| data.min(var) | min(x, na.rm = TRUE) | summarize x, then r(min) |

In R `min(a, b)` on two vectors is one number, the smallest of all their values; the row-by-row
function there is `pmin`. Stata's `min` skips a missing value and returns the other; Zeff, like
`pmin`, returns `NA`.

## Related tasks
- For the smallest value of one variable, the data package has the frame method `min`.
- To hold a value between two limits, use `clamp`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.6 (minimum and maximum
operations).

## See also
max, abs

## Examples
// An Int and a Float give a Float.
> min(3, 7.5)
3

// More than two values.
> min(4, 2, 9)
2

// An unknown value makes the smallest unknown.
> min(1, NA)
NA

// Row by row on the shipped wages: hours capped at 38.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(capped = min(hours, 38))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   capped
  1       19   2012   12.64    29.2       0     29.2
  2       19   2013   25.18    31.4       0     31.4
  3       19   2014   35.49    33.5       1     33.5
  4       19   2015   26.07    58.2       2       38
  5       19   2016      NA      30       3       30
