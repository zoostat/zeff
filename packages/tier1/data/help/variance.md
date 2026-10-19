# variance

## Summary
Sample variance (n − 1)

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

## Details
The corrected two-pass algorithm on deviations scaled by a power of two, so the result is within a
rounding of the exact variance whatever the magnitude of the values, the same on every target.

## References
The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
sd, mean, summary

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.variance(wage)
38.1733
> wages.variance(hours)
12.5
warning: 1 row with NA in hours was skipped (line 3)
