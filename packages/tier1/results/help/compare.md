# compare

## Summary
Compare two nested regressions on the same rows

## Description
Tests the terms the larger of two regressions adds to the smaller, the two fitted on the same
rows and either one nested in the other: the F test of the extra terms under classical errors,
and the Wald F of the extra terms under robust or clustered ones, on the larger fit's degrees of
freedom. The receiver may be the larger or the smaller model. Models on different rows, with
the same terms, or not nested are refused. Like `anova` on two fits in R and Stata's `test` after
the larger model.

## Arguments
- fit: The regression fit.
- other: The other regression fit.

## Value
A `TestResult`, which prints its title, a table of the two models (N, residual sum of squares,
R²) and the statistic line `F(q, df) = …   p = …`.

## Details
Under classical errors F = ((RSS_s − RSS_l) / q) / (RSS_l / df), with RSS_s and RSS_l the
residual sums of squares of the smaller and the larger model, q the terms added and df the
larger model's residual degrees of freedom; under robust or clustered errors the Wald F of the
extra coefficients being zero under the larger model's variance matrix, as `test` computes it.

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
