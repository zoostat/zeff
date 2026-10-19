# median

## Summary
Median

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

## Details
Exact: the values are sorted and the middle taken; the mean of two middle values is one addition
and one halving.

## References
The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
quantile, mean, summary

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.median(wage)
23.4
> wages.median(hours)
37.5
warning: 1 row with NA in hours was skipped (line 3)
