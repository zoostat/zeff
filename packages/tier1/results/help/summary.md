# summary

## Summary
The printed summary of a result as a table

## Description
Gives the summary a result prints, as a `Table` that can be exported: for a fitted model the
title, the lines on the rows used and the standard errors, the table of estimates and the fit
measures; for a test's result its title, its table if it has one, and its statistic line.
`standardized: true` adds a column `beta` after the estimates, each estimate times the standard
deviation of its regressor over that of the outcome, blank for the intercept and for an omitted
term; `fit_measures: true` adds a line with the log-likelihood, AIC and BIC under the fit
measures, which count the variance as a parameter as R's `AIC` and `BIC` do. Like R's `summary`
on a fit and Stata's `regress, beta`.

The summary of a `glm` fit gives its log-likelihood and AIC already, so `fit_measures: true`
adds the BIC alone; `standardized: true` is for a regression and is refused on a `glm` fit
(Z0401), whose coefficients have no one standardization.

`exponentiate: true` gives the odds ratios of a `glm` fit with a logit link in the place of its
coefficients. Each estimate is exp(b), under the heading `odds ratio`: the factor by which the
odds are multiplied when the regressor rises by one. Its standard error is exp(b) times that of
b, by the delta method, and its interval is the interval of b with each end exponentiated, so
it is not symmetric about the ratio. The z and the p-value are those of the coefficient, since
b = 0 is a ratio of 1. The intercept's row is the odds when every regressor is zero. A probit
link and a regression are refused (Z0401): their coefficients are not logarithms of a ratio.
Like Stata's `logit, or`, and R's `exp(coef(m))` with `exp(confint.default(m))`.

## Arguments
- fit: The result.
- standardized: Whether to add the standardized coefficients.
- fit_measures: Whether to add the line of fit measures.
- exponentiate: Whether to give the odds ratios of a `glm` fit with a logit link, exp of each
  coefficient, in the place of the coefficients.

## Value
A `Table`.

## See also
estimates, export, reg

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.summary(standardized: true, fit_measures: true)
reg · log(wage) ~ educ + exper + female
N = 18,179 (233 rows dropped for NA in model variables)
standard errors: classical

  term          estimate      beta   std.error   t value   p value   95% CI
  (intercept)     1.8998                0.0221     86.05    <0.001   [1.8565, 1.9431]
  educ            0.0760    0.3899      0.0014     55.69    <0.001   [0.0733, 0.0787]
  exper           0.0220    0.3417      0.0005     48.78    <0.001   [0.0211, 0.0229]
  female         -0.1329   -0.1371      0.0064    -20.69    <0.001   [-0.1455, -0.1203]

R² 0.203 · adjusted R² 0.203
log-likelihood -10,563.8 · AIC 21,137.5 · BIC 21,176.5

// The odds ratios of a logit: each year of schooling multiplies the odds of union membership by the ratio in its row.
> dataset(wages).glm(union ~ educ + exper + female, family: binomial).summary(exponentiate: true)
glm · union ~ educ + exper + female · binomial, logit
N = 18,390 (22 rows dropped for NA in model variables)
standard errors: classical
odds ratios: exp of the coefficients, standard errors by the delta method; z and p are those of the coefficients

  term          odds ratio   std.error   z value   p value   95% CI
  (intercept)       0.2931      0.0322    -11.16    <0.001   [0.2362, 0.3635]
  educ              1.0009      0.0068      0.14     0.892   [0.9877, 1.0143]
  exper             1.0281      0.0023     12.32    <0.001   [1.0236, 1.0326]
  female            1.0571      0.0337      1.75     0.081   [0.9932, 1.1252]

log-likelihood -11,415.06 · AIC 22,838.13 · McFadden R² 0.007
deviance 22,830.13 on 18,386 df · null deviance 23,001.97 on 18,389 df
converged in 5 iterations of Fisher scoring
