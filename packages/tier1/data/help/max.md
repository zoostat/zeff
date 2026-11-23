# max

## Summary
Largest non-missing value

## In brief
- `d.max(wage)`: the largest value of a variable, one value.
- `d.max(wage, by: year)`: a table with the largest value of each group.
- `d.aggregate(m = max(wage), by: year)` or `d.add(m = max(wage), by: year)`: inside a definition, the largest value of the frame or of each group.

## Description
The max of the non-missing values of a variable, with the variable's type: an Int for an Int
variable, a Float otherwise. Missing values are skipped with a warning giving their count; nothing
left gives `NA`; an infinite value is a value. With `by:` the result is a table with one row per
group; with `weights:` the rows with a zero weight are left out, as section 4.1 of the Language
Reference says, and the weights change nothing else. The max of two or more values row by row is
the `core` function of the same name.

A variable of dates, or of dates and times, gives its latest value, a `Date` or a `DateTime`:
the first and the last day of a file, or of each group with `by:`, and inside `aggregate` the
latest of each group as a variable of that type. The values are compared exactly, as days or
as microseconds.

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float, or a Date or a DateTime.

## Value
An `Int` for an Int variable, a `Date` or a `DateTime` for a variable of dates or times, a
`Float` otherwise; a `Table` with `by:`.

## Missing values
Missing values are skipped, and a warning counts the rows skipped. A variable with no value left gives `NA`, with a warning.

## Details
Exact.

## A common mistake
Writing the variable's name in quotation marks, as R's `d[["wage"]]` does. A variable is named bare:

> dataset(wages).max("wage")
error[Z0401]: `var` of `max` must be a Var, but this is a Str
  = help: name a variable of the frame, without quotes

Write `dataset(wages).max(wage)`.

## Coming from R or Stata
| d.max(wage) | max(d$wage, na.rm = TRUE) | summarize wage, then display r(max) |
| d.max(wage, by: year) | tapply(d$wage, d$year, max, na.rm = TRUE) | tabstat wage, by(year) statistics(max) |

R's `max` gives `NA` when a value is missing unless `na.rm = TRUE`; Stata skips missing values and says nothing; here they are skipped and the warning counts them. R's `max` of no values is `-Inf` with a warning; here it is `NA`. Stata takes a missing value as larger than any number in a comparison, but `summarize` leaves it out, as here.

## Related tasks
- For the smallest value, use `min`.
- For the larger of two values row by row, use the function of `core` that takes two values, also called `max`.
- For the row that holds the largest value, sort with `sort`.
- For the smallest and largest together with other statistics, use `summary`.

## References
The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
summary, median, quantile

## Examples
// The largest value of the wages. The warning counts the rows with no wage.
> let wages = dataset(wages)
> wages.max(wage)
152.39
warning[W0201]: 211 rows with NA in wage were skipped (line 2)

// One row for each group with `by:`.
> wages.max(wage, by: female)
max · wage by female

  female       max
  male     152.390
  female   140.240
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// Of a date, the latest.
> dataset(firms).max(founded)
2009-11-26

// Inside `aggregate`: each worker's last year in the data.
> wages.aggregate(last = max(year), by: worker).head(3)
frame · 3 rows × 2 variables

      worker   last
  1        1   2021
  2        2   2017
  3        3   2012
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 5)
