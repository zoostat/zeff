# max

## Summary
Largest of two or more values, row by row

## Description
The largest of the values given. With Ints the result is an Int; with any Float it is a Float. Any
`NA` among the values gives `NA`, as section 4.1 of the Language Reference has it and as R's `pmax`
does (Stata's `max` skips missing values instead). Given columns, `max` applies row by row, a scalar
standing for its value in every row, and returns a column: `max(wages["hours"], 38)` raises every
value below 38 to 38. The largest value of one column is the frame method `data.max(var)` of the
`data` package.

## Arguments
- a: An Int or a Float, or a column of either.
- b: Another.
- ...: Further values or columns.

## Value
An Int when every value is an Int, a Float otherwise; a column of the same when any argument is a
column.

## Missing values
Any `NA` among the values gives `NA`: the largest of a known and an unknown number is unknown. In columns each row is taken by itself, and nothing is printed.

## Details
Exact. Columns must have the same number of rows (Z0217).

## Coming from R or Stata
| max(a, b) | pmax(a, b) | max(a, b) |
| data.max(var) | max(x, na.rm = TRUE) | summarize x, then r(max) |

In R `max(a, b)` on two vectors is one number, the largest of all their values; the row-by-row
function there is `pmax`. Stata's `max` skips a missing value and returns the other; Zeff, like
`pmax`, returns `NA`.

## Related tasks
- For the largest value of one variable, the data package has the frame method `max`.
- To hold a value between two limits, use `clamp`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.6 (minimum and maximum
operations).

## See also
min, abs

## Examples
// An Int and a Float give a Float.
> max(3, 7.5)
7.5

// More than two values.
> max(4, 2, 9)
9

// An unknown value makes the largest unknown.
> max(1, NA)
NA

// Row by row on the shipped wages: hours raised to at least 38.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(raised = max(hours, 38))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   raised
  1       19   2012   12.64    29.2       0       38
  2       19   2013   25.18    31.4       0       38
  3       19   2014   35.49    33.5       1       38
  4       19   2015   26.07    58.2       2     58.2
  5       19   2016      NA      30       3       38
