# covariance

## Summary
Covariance matrix

## Description
The sample covariances between two or more numeric variables of a frame, as a square table with
the variables as rows and columns, three decimals; the diagonal holds each variable's variance.
`missing:` says how rows with `NA` are dropped: `pairwise` (the default) uses, for each pair, the
rows where both variables have values; `listwise` keeps only the rows where every variable named
has one. The N is printed under the title when every pair has the same, and as a second table,
`N per pair`, when pairwise deletion leaves them different. Like R's `cov()` and Stata's
`correlate, covariance`.

## Arguments
- data: The frame.
- vars: Two or more variables, as a list: `[wage, educ, hours]` or a range. Each must be Bool,
  Int or Float; another type is refused (Z0401), as is a single variable.
- missing: `pairwise` (the default) or `listwise`.

## Value
A `Table`: the covariances, then the N per pair when it varies.

## Details
Each covariance is Σ(x − x̄)(y − ȳ) / (n − 1) over the complete pairs, two-pass with
compensated sums in a fixed order, so the diagonal equals `variance`. Under pairwise deletion a
covariance matrix need not be positive semi-definite, as the matrix of a model should be; `listwise`
gives one that is.

## See also
correlation, variance, sd

## References
- Casella, G. and Berger, R. L. (2002). *Statistical Inference*, 2nd edition, section 4.5.
  Duxbury.

## Examples
> let f = frame(x = [1, 2, 3, 4, 5, 6, 7, 8], y = [2, 1, 4, 3, 7, 5, 8, 6])
> f.covariance([x, y])
covariance · f
N = 8

          x       y
  x   6.000   5.000
  y   5.000   6.000
