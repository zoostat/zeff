# sum

## Summary
Compensated sum of non-missing values

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

## Details
A compensated (Neumaier) sum in row order, so the result is within a rounding of the exact sum and
the same bits on every target.

## References
Neumaier (1974), Rundungsfehleranalyse einiger Verfahren zur Summation endlicher Summen, ZAMM 54,
39–51. The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
mean, n, summary

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.sum(wage)
73.6
> wages.sum(hours)
75
warning: 1 row with NA in hours was skipped (line 3)
