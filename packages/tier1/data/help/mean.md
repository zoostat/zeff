# mean

## Summary
Mean of a variable

## In brief
- `d.mean(wage)`: the mean of a variable, one value.
- `d.mean(wage, by: year)`: a table with the mean of each group.
- `d.mean(wage, weights: hours)`: the weighted form.
- `d.aggregate(m = mean(wage), by: year)` or `d.add(m = mean(wage), by: year)`: inside a definition, the mean of the frame or of each group.

## Description
The arithmetic mean of the non-missing values of `var`. Missing values are skipped with a warning
that gives their count; a column with no value left, or none at all, gives `NA` with a warning. On a Bool
column the mean is the share of `true` values, the commonest summary of a treatment indicator. An
infinite value gives `Inf` or `-Inf`; `Inf` and `-Inf` together give `NA` with a warning.

Unlike R's `mean`, which needs `na.rm = TRUE`, missing values never make the result `NA` silently:
they are skipped and counted. With `by:` the result is a table with one row per group, in key
order with `NA` last; with `weights:` the mean is Σwx / Σw, whatever `weight_type:` says. Inside
`add` the same function is a reducer: `add(dev = wage - mean(wage), by: region)` takes group means.

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.

## Value
A `Float`; a `Table` with `by:`.

## Missing values
Missing values are skipped, and a warning counts the rows skipped. A variable with no value left gives `NA`, with a warning.

## Details
The sum is a compensated (Neumaier) sum taken in row order, divided by the count and corrected once
by the mean of the deviations, so the result is within a rounding of the exact mean whatever the
magnitude of the values, and the same bits on every target.

## A common mistake
Giving an expression where the variable goes. The variable is named, and a new one is made first:

> dataset(wages).mean(log(wage))
error[Z0401]: `var` of `mean` takes a variable of the frame, not an expression
  = help: compute it first and name the new variable: `.add(v = log(wage))` before `.mean(v)`

Make the variable, then name it: `dataset(wages).add(lwage = log(wage)).mean(lwage)`.

## Coming from R or Stata
| d.mean(wage) | mean(d$wage, na.rm = TRUE) | summarize wage, then display r(mean) |
| d.mean(wage, by: year) | tapply(d$wage, d$year, mean, na.rm = TRUE) | tabstat wage, by(year) statistics(mean) |

R's `mean` gives `NA` when a value is missing unless `na.rm = TRUE`; Stata skips missing values and says nothing; here they are skipped and the warning counts them. With `weights:` R's form is `weighted.mean(d$wage, d$hours, na.rm = TRUE)` and Stata's `summarize wage [aweight = hours]`.

## Related tasks
- For the spread about the mean, use `sd`.
- For the middle value, which a few extreme values do not move, use `median`.
- For the usual statistics of several variables at once, use `summary`.
- To test a mean against a value or another mean, use `t_test`.

## References
Neumaier (1974), Rundungsfehleranalyse einiger Verfahren zur Summation endlicher Summen, ZAMM 54, 39–51.
Higham (2002), Accuracy and Stability of Numerical Algorithms, second edition, chapter 4.

## See also
sd, summary, describe

## Examples
// The mean of the wages. The warning counts the rows with no wage.
> let wages = dataset(wages)
> wages.mean(wage)
27.3695
warning[W0201]: 211 rows with NA in wage were skipped (line 2)

// One row for each group with `by:`.
> wages.mean(wage, by: female)
mean · wage by female

  female     mean
  male     29.189
  female   25.437
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// Weighted by the hours worked.
> wages.mean(wage, weights: hours)
27.433
warning[W0201]: 35 rows with NA in hours were dropped (line 4)
warning[W0201]: 210 rows with NA in wage were skipped (line 4)

// Of a variable of true and false, or of 0 and 1, the mean is the share: here the share of union members.
> wages.mean(union)
0.318053

// Inside `add`, the mean of each group on every row of the group: each wage less the mean of its year.
> wages.add(dev = wage - mean(wage), by: year).select([worker, year, wage, dev]).head(3)
frame · 3 rows × 4 variables · panel worker × year

      worker   year    wage        dev
  1        1   2015   18.08   -8.93159
  2        1   2016   17.28   -10.5654
  3        1   2017   20.93   -7.51246
warning[W0201]: 211 rows with NA in wage were skipped (line 6)
