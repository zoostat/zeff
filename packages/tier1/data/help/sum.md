# sum

## Summary
Compensated sum of non-missing values

## In brief
- `d.sum(wage)`: the sum of a variable, one value.
- `d.sum(wage, by: year)`: a table with the sum of each group.
- `d.sum(wage, weights: hours)`: the weighted form.
- `d.aggregate(m = sum(wage), by: year)` or `d.add(m = sum(wage), by: year)`: inside a definition, the sum of the frame or of each group.

## Description
The sum of the non-missing values of a variable, with the variable's type: an Int for an Int
variable (an overflow is an error, Z0211, never a wrong number), a Float otherwise; on a Bool
variable, the count of `true` values. Missing values are skipped with a warning giving their count,
and a variable with nothing left gives `NA` with a warning, where R and Stata give 0. With `by:` the
result is a table with one row per group; with `weights:` it is the weighted total Σwx under
frequency and sampling weights and, under analytic weights, that total scaled by n / Σw.

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.

## Value
An `Int` for an Int variable, a `Float` otherwise; a `Table` with `by:`.

## Missing values
Missing values are skipped, and a warning counts the rows skipped. A variable with no value left gives `NA`, with a warning, where R and Stata give 0.

## Details
A compensated (Neumaier) sum in row order, so the result is within a rounding of the exact sum and
the same bits on every target.

## A common mistake
Asking it of a variable that is not a number. A category or a text has no such statistic:

> dataset(firms).sum(industry)
error[Z0401]: `sum` needs a Bool, Int or Float, but `industry` is Category
  = help: choose a Bool, Int or Float variable; `firms.describe()` lists the types

Name a variable of numbers, `dataset(firms).sum(size)`; for the counts of a category's levels, use `tabulate`.

## Coming from R or Stata
| d.sum(wage) | sum(d$wage, na.rm = TRUE) | summarize wage, then display r(sum) |
| d.sum(wage, by: year) | tapply(d$wage, d$year, sum, na.rm = TRUE) | tabstat wage, by(year) statistics(sum) |

R's `sum` gives `NA` when a value is missing unless `na.rm = TRUE`; Stata skips missing values and says nothing; here they are skipped and the warning counts them. Of no values at all R and Stata give 0; here the sum of nothing is `NA`, so that an empty group is not taken for a group whose total is nought.

## Related tasks
- For the sum divided by the count, use `mean`.
- For the number of rows, use `n_rows`, or `n` inside a definition.
- For a sum so far down the rows, use `running`.
- For the sum across several variables in each row, use `row`.

## References
Neumaier (1974), Rundungsfehleranalyse einiger Verfahren zur Summation endlicher Summen, ZAMM 54,
39–51. The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
mean, n, summary

## Examples
// The sum of the wages. The warning counts the rows with no wage.
> let wages = dataset(wages)
> wages.sum(wage)
498153
warning[W0201]: 211 rows with NA in wage were skipped (line 2)

// One row for each group with `by:`.
> wages.sum(wage, by: female)
sum · wage by female

  female          sum
  male     273646.150
  female   224506.790
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// Weighted: the total of the wages, each counted by the hours worked.
> wages.sum(wage, weights: hours)
18911277
warning[W0201]: 35 rows with NA in hours were dropped (line 4)
warning[W0201]: 210 rows with NA in wage were skipped (line 4)

// Of a variable of 0 and 1, the count of ones: the rows of union members.
> wages.sum(union)
5856

// Inside `add`: each row's share of the total of its group.
> wages.add(share = hours / sum(hours), by: year).select([worker, year, hours, share]).head(3)
frame · 3 rows × 4 variables · panel worker × year

      worker   year   hours         share
  1        1   2015    45.1   0.000543856
  2        1   2016    41.4   0.000510445
  3        1   2017    54.3   0.000704362
warning[W0201]: 35 rows with NA in hours were skipped (line 6)
