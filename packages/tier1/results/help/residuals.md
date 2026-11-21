# residuals

## Summary
The residuals of a fitted model

## Description
Gives the residuals, outcome less fitted value, as a column of the fitted frame's full length
with `NA` in the rows the fit dropped, so that `wages.add(res = m.residuals())` lines up, as R's
`na.exclude` and Stata's `predict, residuals` do. Under weights the residuals are those of the
data, not of the weighted rows. A test's result has none and gives `NA`.

A `glm` fit has four kinds, chosen by `type:`. `response` is the outcome less the fitted
probability, the kind every model has, and the one given unless another is asked for.
`pearson` divides it by the standard deviation the model gives the outcome, `deviance` is the
signed square root of the row's part of the deviance, and `working` is the residual of the last
weighted least-squares step. R's `residuals` gives the deviance residuals of a `glm` fit unless
told otherwise; here they are `type: deviance`.

## Arguments
- fit: The result.
- type: The kind of residual of a model fitted by maximum likelihood: `response`, `pearson`, `deviance` or `working`; `response` unless given. A regression has the first alone, and another word is refused (Z0401).

## Value
A `Column<Float>` of the fitted frame's length.

## Details
The residuals are y − x′β computed row by row in a second pass over the data after the fit,
in the data's units under any weighting; the fit checks them orthogonal to the regressors
before it returns.

For a fit by maximum likelihood with mean μ, variance function V and weight w in each row:
`response` is y − μ; `pearson` is (y − μ)√w/√V(μ), whose squares sum to Pearson's chi-squared;
`deviance` is the square root of the row's part of the deviance with the sign of y − μ, whose
squares sum to the deviance; `working` is (y − μ)/(dμ/dη). For a binary outcome V(μ) = μ(1 − μ).
The weights are those the fit used: frequency weights as given, sampling weights scaled to sum
to the rows.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapter 3.

McCullagh, P. and Nelder, J. A. (1989). *Generalized Linear Models*, 2nd edition. Chapman and
Hall, section 2.4.

## See also
fitted, predict, influence, glm

## Examples
> let wages = dataset(wages)
> let m = wages.reg(log(wage) ~ educ + exper + female)
> wages.add(res = m.residuals()).summary([res])
summary · frame

  variable        N   missing    mean      sd      min     max
  res        18,179       233   0.000   0.433   -2.006   1.530
warning[W0201]: 233 rows with NA in res were skipped (line 3)
> let g = wages.glm(union ~ educ + exper + female, family: binomial)
> wages.add(dev = g.residuals(type: deviance)).summary([dev])
summary · frame

  variable        N   missing     mean      sd      min     max
  dev        18,390        22   -0.114   1.108   -1.108   1.718
warning[W0201]: 22 rows with NA in dev were skipped (line 5)
