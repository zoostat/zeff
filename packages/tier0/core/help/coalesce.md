# coalesce

## Summary
First non-missing value

## Description
The first of its arguments that is not `NA`, or `NA` when all of them are. The arguments must be of
one type, Int and Float joining as Float (section 3.7 of the Language Reference, "Joining types"):
`coalesce(NA, 2)` is `2` and `coalesce(NA, NA, "none")` is `none`, while `coalesce(1, "a")` is
refused (Z0214). Given columns, `coalesce` applies row by row, a scalar standing for its value in
every row, and returns a column: `coalesce(wages["hours"], 0)` fills the gaps in hours with zero, and
`coalesce(a, b)` takes `b` wherever `a` is missing, as dplyr's `coalesce` does.

## Arguments
- a: A value of a computable type (Bool, Int, Float or Str), or a column of one.
- b: Another of the same type.
- ...: Further values or columns of the same type.

## Value
A value of the arguments' joined type; a column of it when any argument is a column.

## Missing values
It is the function for them: each `NA` of the first argument is replaced by the value of the next that has one, and the result is `NA` only where every argument is.

## Details
Exact. Columns must have the same number of rows (Z0217).

## A common mistake
Filling the gaps of a number with a text, such as the word "unknown". The result would hold two types, and the checker refuses it:

> coalesce(1, "a")
error[Z0214]: `coalesce` needs values of one type, but these are an Int and a Str
  = help: Int and Float join as Float, and NA fits any type; convert with `to` where a mixture is meant

Fill with a value of the same type, `coalesce(x, 0)`, or convert first, `coalesce(to(x, Str), "unknown")`.

## Coming from R or Stata
| coalesce(a, b) | dplyr::coalesce(a, b), or ifelse(is.na(a), b, a) | cond(missing(a), b, a) |
| coalesce(x, 0) | tidyr::replace_na(x, 0) | replace x = 0 if missing(x) |

Base R has no single function. `dplyr::coalesce` refuses two types as well.

## Related tasks
- To test for a missing value, use `is_missing`.
- To fill gaps from the rows before or after, use `fill_missing`.

## References
Language Reference, sections 3.7 ("Joining types") and 4.1.

## See also
is_missing, min, max

## Examples
// The first value that is known.
> coalesce(NA, 2)
2

// More than two.
> coalesce(NA, NA, "none")
none

// Row by row on the shipped wages: the missing wage filled with zero.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(filled = coalesce(wage, 0))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   filled
  1       19   2012   12.64    29.2       0    12.64
  2       19   2013   25.18    31.4       0    25.18
  3       19   2014   35.49    33.5       1    35.49
  4       19   2015   26.07    58.2       2    26.07
  5       19   2016      NA      30       3        0

// One variable's gaps filled from another.
> w.add(either = coalesce(wage, hours))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   either
  1       19   2012   12.64    29.2       0    12.64
  2       19   2013   25.18    31.4       0    25.18
  3       19   2014   35.49    33.5       1    35.49
  4       19   2015   26.07    58.2       2    26.07
  5       19   2016      NA      30       3       30
