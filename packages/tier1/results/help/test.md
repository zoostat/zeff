# test

## Summary
Wald test of linear restrictions on the coefficients

## Description
Tests one or more restrictions on the coefficients of a regression together, each written as an
equality over the model's terms: `m.test(educ == exper)`, `m.test(educ + exper == 1, female == 0)`.
Inside the call a bare name is a term of the model, never a column or a binding, and a term
whose name is not a plain identifier is written in backticks, `` `exper^2` ``. The test is the
Wald F under the fit's own variance matrix, so robust or clustered errors carry through, with the
fit's degrees of freedom. A restriction that is not linear in the coefficients is refused and
pointed to `derive`. Like Stata's `test` and `car::linearHypothesis` in R.

## Arguments
- fit: The regression fit.
- ...: The restrictions, each an equality over the terms.

## Value
A `TestResult`, which prints its title, the statistic line `F(q, df) = …   p = …`, and answers
`p_values()` with the p-value.

## Details
With R the q × k matrix of the restrictions and c their constants, the statistic is
(Rβ − c)ᵀ (R V Rᵀ)⁻¹ (Rβ − c) / q, with V the fit's variance matrix, referred to F on (q, df),
df the fit's residual degrees of freedom (G − 1 under clustered errors); a single restriction
gives the square of the term's t statistic. The restrictions must be independent; ones that
repeat another are refused.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapter 6.

## See also
compare, derive, coef, vcov

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.test(educ == exper)
test · m · educ == exper
F(1, 18,175) = 1748.67   p < 0.001
> m.test(educ + exper == 0.1, female == -0.1)
test · m · educ + exper == 0.1, female == -0.1
F(2, 18,175) = 13.88   p < 0.001
