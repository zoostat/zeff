# summary

## Summary
Table of N, missing, mean, SD, min and max

## Description
One row per variable with its count of non-missing values (`N`), its count of missing values, and
its mean, sample standard deviation (n − 1), minimum and maximum, computed as `mean` and `sd`
compute them: missing values are skipped and the rows skipped are counted in one warning under the
table. Without `vars`, every Bool, Int or Float variable is summarised, in the frame's order; a
Bool variable counts `true` as 1. `stats:` chooses the statistics and their order from the six
words `n`, `missing`, `mean`, `sd`, `min` and `max`; any other word is refused.

Statistics print with three decimals, as section 10.2 of the Language Reference prints tables;
`mean` and `sd` on their own print six significant digits. A variable whose values are all missing
shows `NA` throughout, with a warning; one value alone has an `NA` standard deviation, and so has
a variable holding `Inf`.

Like R's `summary` on numeric columns and Stata's `summarize`, with the count of missing values
beside the count of observations.

## Arguments
- data: The frame.
- vars: The variables to summarise, in a list, as in `[wage, hours]`; one may be written alone.
  Omitted, every Bool, Int or Float variable.
- stats: The statistics to show, as words in a list, in the order wanted: `stats: [mean, sd]`.
  Omitted, all six in the order N, missing, mean, sd, min, max.

## Value
A `Table`, which prints as shown and can be passed to `print`.

## Details
The mean is a compensated sum corrected once; the standard deviation is the corrected two-pass
algorithm with the deviations scaled before squaring; see `mean` and `sd`. Minimum and maximum are
taken over the non-missing values, so `Inf` and `-Inf` appear as themselves.

## References
Chan, Golub and LeVeque (1983), Algorithms for computing the sample variance: analysis and
recommendations, The American Statistician 37, 242–247.

## See also
mean, sd, describe

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.summary([wage, hours])
summary · wages

  variable   N   missing     mean      sd      min      max
  wage       3         0   24.533   6.178   19.000   31.200
  hours      2         1   37.500   3.536   35.000   40.000
warning: 1 row with NA in hours was skipped (line 2)
> wages.summary(stats: [n, mean])
summary · wages

  variable   N     mean
  id         3    2.000
  wage       3   24.533
  hours      2   37.500
warning: 1 row with NA in hours was skipped (line 3)
