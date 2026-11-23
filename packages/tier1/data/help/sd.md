# sd

## Summary
Sample standard deviation

## In brief
- `d.sd(wage)`: the standard deviation of a variable, one value.
- `d.sd(wage, by: year)`: a table with the standard deviation of each group.
- `d.sd(wage, weights: hours)`: the weighted form.
- `d.aggregate(m = sd(wage), by: year)` or `d.add(m = sd(wage), by: year)`: inside a definition, the standard deviation of the frame or of each group.

## Description
The sample standard deviation of the non-missing values of `var`: the square root of the sum of
squared deviations from the mean divided by n − 1. Missing values are skipped with a warning that
gives their count. One value alone has no spread, so a column with one value left gives `NA` with
a warning, as does a column holding `Inf` or `-Inf`; a constant column gives `0`. On a Bool column
the values are 1 for `true` and 0 for `false`.

Like R's `sd` and Stata's `summarize`, which divide by n − 1; unlike R, missing values never make
the result `NA` silently.

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.

## Value
A `Float`; a `Table` with `by:`.

## Missing values
Missing values are skipped, and a warning counts the rows skipped. One value alone has no spread: a variable with fewer than two values left gives `NA`, with a warning.

## Details
The corrected two-pass algorithm: the mean first (a compensated sum, corrected once), then the
compensated sum of squared deviations from it, less the correction for the mean's own rounding
error, divided by n − 1. The result is never negative, so a constant column gives exactly `0`, and
the same bits on every target.

## A common mistake
Asking it of a variable that is not a number. A category or a text has no such statistic:

> dataset(firms).sd(industry)
error[Z0401]: `sd` needs a Bool, Int or Float, but `industry` is Category
  = help: choose a Bool, Int or Float variable; `firms.describe()` lists the types

Name a variable of numbers, `dataset(firms).sd(size)`; for the counts of a category's levels, use `tabulate`.

## Coming from R or Stata
| d.sd(wage) | sd(d$wage, na.rm = TRUE) | summarize wage, then display r(sd) |
| d.sd(wage, by: year) | tapply(d$wage, d$year, sd, na.rm = TRUE) | tabstat wage, by(year) statistics(sd) |

R's `sd` gives `NA` when a value is missing unless `na.rm = TRUE`; Stata skips missing values and says nothing; here they are skipped and the warning counts them. All three divide by n − 1.

## Related tasks
- For the variance, its square, use `variance`.
- For a spread that extreme values do not move, use `iqr` or `mad`.
- For the centre, use `mean`.
- To put a variable on the scale of its standard deviation, use `standardize`.

## References
Chan, Golub and LeVeque (1983), Algorithms for computing the sample variance: analysis and
recommendations, The American Statistician 37, 242–247.
Higham (2002), Accuracy and Stability of Numerical Algorithms, second edition, chapter 1.

## See also
mean, summary, describe

## Examples
// The standard deviation of the wages. The warning counts the rows with no wage.
> let wages = dataset(wages)
> wages.sd(wage)
14.0269
warning[W0201]: 211 rows with NA in wage were skipped (line 2)

// One row for each group with `by:`.
> wages.sd(wage, by: female)
sd · wage by female

  female       sd
  male     14.787
  female   12.895
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// Weighted by the hours worked.
> wages.sd(wage, weights: hours)
14.0549
warning[W0201]: 35 rows with NA in hours were dropped (line 4)
warning[W0201]: 210 rows with NA in wage were skipped (line 4)

// Inside `aggregate`, beside other statistics of each group.
> wages.aggregate(mean = mean(wage), sd = sd(wage), by: union)
frame · 2 rows × 3 variables

      union      mean        sd
  1   no       26.463   13.6062
  2   yes     29.3138   14.7032
warning[W0201]: 211 rows with NA in wage were skipped (line 5)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 5)
