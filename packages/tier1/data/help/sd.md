# sd

## Summary
Sample standard deviation

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
A `Float`.

## Details
The corrected two-pass algorithm: the mean first (a compensated sum, corrected once), then the
compensated sum of squared deviations from it, less the correction for the mean's own rounding
error, divided by n − 1. The result is never negative, so a constant column gives exactly `0`, and
the same bits on every target.

## References
Chan, Golub and LeVeque (1983), Algorithms for computing the sample variance: analysis and
recommendations, The American Statistician 37, 242–247.
Higham (2002), Accuracy and Stability of Numerical Algorithms, second edition, chapter 1.

## See also
mean, summary, describe

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.sd(wage)
6.17846
> wages.sd(hours)
3.53553
warning: 1 row with NA in hours was skipped (line 3)
