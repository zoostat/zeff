# compare

## Summary
Compare two nested regressions on the same rows

## Description
Tests the terms the larger of two regressions adds to the smaller, the two fitted on the same
rows and either one nested in the other: the F test of the extra terms under classical errors,
and the Wald F of the extra terms under robust or clustered ones, on the larger fit's degrees of
freedom. The receiver may be the larger or the smaller model. Models on different rows, with
the same terms, or not nested are refused; so are two models of different outcomes, two fitted
to different data that happen to have as many rows, and two under different weights. Like
`anova` on two fits in R and Stata's `test` after the larger model.

Two `glm` fits are compared by the likelihood-ratio test: twice the gap between their
log-likelihoods, referred to chi-squared on as many degrees of freedom as terms added. It needs
the same family and link, the same rows and weights, and model-based standard errors on both:
under robust or clustered errors, or sampling weights, it is refused (Z0416), and `test` gives
the Wald test of the extra terms. Like `anova(m0, m1, test = "LRT")` in R and Stata's `lrtest`.

## Arguments
- fit: The regression fit.
- other: The other regression fit.

## Value
A `TestResult`, which prints its title, a table of the two models (N, residual sum of squares,
R²) and the statistic line `F(q, df) = …   p = …`; for two `glm` fits, the table gives N, the
log-likelihood and the deviance of each, and the line is `chi2(q) = …   p = …`.

## Details
Under classical errors F = ((RSS_s − RSS_l) / q) / (RSS_l / df), with RSS_s and RSS_l the
residual sums of squares of the smaller and the larger model, q the terms added and df the
larger model's residual degrees of freedom; under robust or clustered errors the Wald F of the
extra coefficients being zero under the larger model's variance matrix, as `test` computes it.

For two fits by maximum likelihood the statistic is 2(ℓ_l − ℓ_s), the difference of the two
deviances, on chi-squared with q degrees of freedom.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapters 4 and 6.

## See also
test, derive, reg

## Examples
> let wages = dataset(wages)
> let m1 = wages.reg(log(wage) ~ educ + exper)
> let m2 = wages.reg(log(wage) ~ educ + exper + female)
> m2.compare(m1)
compare · m2 against m1

  model        N       RSS      R²
  m2      18,179   3402.75   0.203
  m1      18,179   3482.91   0.184

F(1, 18,175) = 428.19   p < 0.001
> let g1 = wages.glm(union ~ educ + exper, family: binomial)
> let g2 = wages.glm(union ~ educ + exper + female, family: binomial)
> g2.compare(g1)
compare · g2 against g1

  model        N   log-likelihood    deviance
  g2      18,390       -11,415.06   22,830.13
  g1      18,390       -11,416.59   22,833.17

chi2(1) = 3.05   p = 0.081
