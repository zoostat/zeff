# median

## Summary
Median

## In brief
- `d.median(wage)`: the median of a variable, one value.
- `d.median(wage, by: year)`: a table with the median of each group.
- `d.median(wage, weights: hours)`: the weighted form.
- `d.aggregate(m = median(wage), by: year)` or `d.add(m = median(wage), by: year)`: inside a definition, the median of the frame or of each group.

## Description
The median of the non-missing values of `var`: the middle value, or the mean of the two middle values
for an even count (R's `median`, the type-7 quantile at one half). Missing values are skipped with a
warning giving their count; nothing left gives `NA`. With `by:` the result is a table with one row
per group; with `weights:` it is the weighted median, the smallest value whose cumulative weight
reaches half the total (for frequency weights, the median of the replicated rows under the
type-1 definition).

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.

## Value
A `Float`; a `Table` with `by:`.

## Missing values
Missing values are skipped, and a warning counts the rows skipped. A variable with no value left gives `NA`, with a warning.

## Details
Exact: the values are sorted and the middle taken; the mean of two middle values is one addition
and one halving.

## A common mistake
Giving an expression where the variable goes. The variable is named, and a new one is made first:

> dataset(wages).median(log(wage))
error[Z0401]: `var` of `median` takes a variable of the frame, not an expression
  = help: compute it first and name the new variable: `.add(v = log(wage))` before `.median(v)`

Make the variable, then name it: `dataset(wages).add(lwage = log(wage)).median(lwage)`.

## Coming from R or Stata
| d.median(wage) | median(d$wage, na.rm = TRUE) | summarize wage, detail, then display r(p50) |
| d.median(wage, by: year) | tapply(d$wage, d$year, median, na.rm = TRUE) | tabstat wage, by(year) statistics(median) |

R's `median` gives `NA` when a value is missing unless `na.rm = TRUE`; Stata skips missing values and says nothing; here they are skipped and the warning counts them. For an even count all three take the mean of the two middle values.

## Related tasks
- For any other quantile, use `quantile`.
- For the mean, use `mean`.
- For the spread about the median, use `iqr` or `mad`.
- To test the medians of two groups, use `wilcoxon_test`.

## References
The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
quantile, mean, summary

## Examples
// The median of the wages. The warning counts the rows with no wage.
> let wages = dataset(wages)
> wages.median(wage)
24.35
warning[W0201]: 211 rows with NA in wage were skipped (line 2)

// One row for each group with `by:`.
> wages.median(wage, by: female)
median · wage by female

  female   median
  male     25.900
  female   22.680
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// Weighted by the hours worked.
> wages.median(wage, weights: hours)
24.4
warning[W0201]: 35 rows with NA in hours were dropped (line 4)
warning[W0201]: 210 rows with NA in wage were skipped (line 4)

// The median is below the mean where a few values are large, as with wages.
> wages.aggregate(median = median(wage), mean = mean(wage))
frame · 1 row × 2 variables

      median      mean
  1    24.35   27.3695
warning[W0201]: 211 rows with NA in wage were skipped (line 5)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 5)
