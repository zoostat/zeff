# reg

## Summary
Linear regression by ordinary least squares

## Description
Fits the formula by ordinary least squares on the rows of the frame without `NA` in a model
variable, which are dropped and counted, and prints the summary of section 10.2 of the Language
Reference: the formula, the rows used, the kind of standard error, a table with each term's
estimate, standard error, t value, p-value and confidence interval, and R² with its adjusted
value. The formula's terms follow section 7.2: `a * b` expands, a category gets its indicator
columns by its contrast with the first level as the base unless `base()` names another, `exper^2`
and `log(wage)` are computed before the fit, and a term written twice is kept once with a note. A
regressor collinear with the columns before it is dropped and named in a note; a design whose
scaled condition number passes 1e8 is warned about (W1001). Standard errors are classical at this
release; robust and clustered errors, weights and fixed effects follow in 0.4.0 and 0.6.0. Like
R's `lm` and Stata's `regress`.

## Arguments
- data: The frame.
- formula: `y ~ terms`, written in the call or bound with `let`.
- se: The kind of standard error: `classical`, `robust` (HC1), `hc2` or `hc3`; `classical` unless
  sampling weights are given, when `robust`.
- cluster: The variable whose values group the observations for clustered errors (CR1).
- weights: The weight variable.
- weight_type: What the weights are: `frequency` counts, `analytic` precision weights, `sampling`
  probability weights.
- level: The confidence level of the intervals.

## Value
A `RegFit`, which prints as its summary.

## Details
Each regressor is scaled to unit norm before a blocked Householder QR of the design and the
outcome together, so that the fit of a column at 1e12 is as accurate as one at 1, and no n × k
matrix is held; the k × k R factor is then factorised again with column pivoting, a column whose
remaining norm is below 1e-7 of its own being dropped as collinear with those before it, which is
R's rule. The estimates come from back substitution and are unscaled; residuals, fitted values
and leverages come from a second pass over the rows. The classical variance is the residual sum
of squares over n − k times (XᵀX)⁻¹, read from R; `robust` is HC1, the sandwich of the scores
scaled by n/(n − k), as Stata's `robust` and R's `sandwich::vcovHC(type = "HC1")`; `hc2` and
`hc3` scale each observation's score by 1/(1 − h) and 1/(1 − h)², with h its leverage; `cluster:`
sums the scores within each cluster (CR1) and applies Stata's adjustment G/(G − 1) · (n − 1)/(n −
k), with the t table on G − 1 degrees of freedom, as Stata's `regress` and reghdfe do and as
`sandwich::vcovCL(type = "HC1")` gives; the other kinds use n − k. Weights follow section 6:
frequency weights count each row as many times as its weight, so N is their sum; analytic weights
are precision weights, scaled to sum to the rows; sampling weights are probability weights, with
which `se:` defaults to `robust` and `classical` is refused. A row with `NA` in the weight or the
cluster is dropped with the others; a zero weight leaves its row out, with a note; a negative
weight is an error. After every fit the residuals are checked to be orthogonal to the regressors
and the variance matrix to be symmetric with no negative variance, a failure being an internal
error rather than a result.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapters 2 and 3. MacKinnon, J. G. and White, H. (1985). Some heteroskedasticity-consistent
covariance matrix estimators with improved finite sample properties. *Journal of Econometrics*,
29, 305–325. Cameron, A. C. and Miller, D. L. (2015). A practitioner's guide to cluster-robust
inference. *Journal of Human Resources*, 50, 317–372.

## See also
category, base, interact, summary

## Examples
> let wages = dataset(wages)
> wages.reg(log(wage) ~ educ + exper + exper^2 + female)
reg · log(wage) ~ educ + exper + exper^2 + female
N = 18,179 (233 rows dropped for NA in model variables)
standard errors: classical

  term          estimate   std.error   t value   p value   95% CI
  (intercept)     1.7873      0.0236     75.74    <0.001   [1.7410, 1.8335]
  educ            0.0758      0.0014     55.80    <0.001   [0.0731, 0.0785]
  exper           0.0421      0.0016     26.36    <0.001   [0.0389, 0.0452]
  exper^2        -0.0007      0.0001    -13.09    <0.001   [-0.0008, -0.0006]
  female         -0.1319      0.0064    -20.63    <0.001   [-0.1445, -0.1194]

R² 0.210 · adjusted R² 0.210
