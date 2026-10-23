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

## Arguments
- fit: The result.
- standardized: Whether to add the standardized coefficients.
- fit_measures: Whether to add the line of fit measures.

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
