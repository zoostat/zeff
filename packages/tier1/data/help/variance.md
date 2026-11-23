# variance

## Summary
Sample variance (n − 1)

## In brief
- `d.variance(wage)`: the variance of a variable, one value.
- `d.variance(wage, by: year)`: a table with the variance of each group.
- `d.variance(wage, weights: hours)`: the weighted form.
- `d.aggregate(m = variance(wage), by: year)` or `d.add(m = variance(wage), by: year)`: inside a definition, the variance of the frame or of each group.

## Description
The sample variance of the non-missing values of `var`, with divisor n − 1, as R's `var`. Missing
values are skipped with a warning that gives their count; fewer than two values, or an infinite one,
make it undefined, which is `NA` with a warning (W0205); a constant variable gives 0. With `by:` the
result is a table with one row per group, in key order; with `weights:` the variance is weighted as
`weight_type:` says: frequency weights replicate rows, analytic and sampling weights are scaled to
sum to n with divisor n − 1, as Stata's `summarize [aweight]`.

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.

## Value
A `Float`; a `Table` with `by:`.

## Missing values
Missing values are skipped, and a warning counts the rows skipped. A variable with fewer than two values left gives `NA`, with a warning.

## Details
The corrected two-pass algorithm on deviations scaled by a power of two, so the result is within a
rounding of the exact variance whatever the magnitude of the values, the same on every target.

## A common mistake
Writing the variable's name in quotation marks, as R's `d[["wage"]]` does. A variable is named bare:

> dataset(wages).variance("wage")
error[Z0401]: `var` of `variance` must be a Var, but this is a Str
  = help: name a variable of the frame, without quotes

Write `dataset(wages).variance(wage)`.

## Coming from R or Stata
| d.variance(wage) | var(d$wage, na.rm = TRUE) | summarize wage, then display r(Var) |
| d.variance(wage, by: year) | tapply(d$wage, d$year, var, na.rm = TRUE) | tabstat wage, by(year) statistics(variance) |

R's `var` gives `NA` when a value is missing unless `na.rm = TRUE`; Stata skips missing values and says nothing; here they are skipped and the warning counts them. All three divide by n − 1.

## Related tasks
- For its square root, in the units of the variable, use `sd`.
- To compare the variances of groups, use `variance_test`.
- For the covariances of several variables, use `covariance`.

## References
The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
sd, mean, summary

## Examples
// The variance of the wages. The warning counts the rows with no wage.
> let wages = dataset(wages)
> wages.variance(wage)
196.755
warning[W0201]: 211 rows with NA in wage were skipped (line 2)

// One row for each group with `by:`.
> wages.variance(wage, by: female)
variance · wage by female

  female   variance
  male      218.646
  female    166.273
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// Weighted by the hours worked.
> wages.variance(wage, weights: hours)
197.541
warning[W0201]: 35 rows with NA in hours were dropped (line 4)
warning[W0201]: 210 rows with NA in wage were skipped (line 4)

// It is the square of the standard deviation.
> wages.sd(wage)^2
196.755
warning[W0201]: 211 rows with NA in wage were skipped (line 5)
