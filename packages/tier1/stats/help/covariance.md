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

## Missing values
With `missing: pairwise`, the default, each covariance is computed on the rows where both of its variables have a value, and the table `N per pair` shows how many when the pairs differ. With `missing: listwise` a row with `NA` in any variable named is left out of every pair. No note is printed.

## Details
Each covariance is Σ(x − x̄)(y − ȳ) / (n − 1) over the complete pairs, two-pass with
compensated sums in a fixed order, so the diagonal equals `variance`. Under pairwise deletion a
covariance matrix need not be positive semi-definite, as the matrix of a model should be; `listwise`
gives one that is.

## A common mistake
Naming two variables one after the other, as `cov(x, y)` in R. The variables go in one list, and the place after it is `missing:`, so the second name is read as its word:

> dataset(wages).covariance(wage, educ)
error[Z0402]: `educ` is not one of the allowed words

Write the variables as a list: `covariance([wage, educ])`.

## Coming from R or Stata
| d.covariance([x, y, z]) | cov(d[, c("x", "y", "z")], use = "pairwise.complete.obs") | correlate x y, covariance, a pair at a time |
| d.covariance([x, y, z], missing: listwise) | cov(d[, c("x", "y", "z")], use = "complete.obs") | correlate x y z, covariance |

All three divide by n − 1. R's `cov` gives `NA` for a pair as soon as one of its variables has
a missing value, unless `use =` is written. Stata's `correlate` keeps the rows that are
complete in every variable named, which is `missing: listwise` here.

## Related tasks
- For the same on a scale from −1 to 1, use `correlation`.
- For the variance of one variable, use `variance`.
- For the variance matrix of the coefficients of a fit, use `vcov`.
- For the slope of one variable on another, use `reg`.

## References
- Casella, G. and Berger, R. L. (2002). *Statistical Inference*, 2nd edition, section 4.5.
  Duxbury.

## See also
correlation, variance, sd

## Examples
// The covariances of wage, schooling and hours in one year; the diagonal holds the variances.
> let w = dataset(wages).where(year == 2019)
> w.covariance([wage, educ, hours])
covariance · w

             wage     educ    hours
  wage    210.498   11.246    0.726
  educ     11.246    6.508   -0.556
  hours     0.726   -0.556   65.696

N per pair

           wage    educ   hours
  wage    1,492   1,492   1,489
  educ    1,492   1,518   1,515
  hours   1,489   1,515   1,515

// On the rows complete in all three, with `missing: listwise`.
> w.covariance([wage, educ, hours], missing: listwise)
covariance · w
N = 1,489

             wage     educ    hours
  wage    210.665   11.232    0.726
  educ     11.232    6.508   -0.563
  hours     0.726   -0.563   65.862

// Two short variables, with the default named.
> let f = frame(x = [1, 2, 3, 4, 5, 6, 7, 8], y = [2, 1, 4, 3, 7, 5, 8, 6])
> f.covariance([x, y], missing: pairwise)
covariance · f
N = 8

          x       y
  x   6.000   5.000
  y   5.000   6.000
