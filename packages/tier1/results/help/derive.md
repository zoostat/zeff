# derive

## Summary
Nonlinear combinations of the estimates with delta-method standard errors

## Description
Computes quantities defined from a regression's coefficients, such as a ratio of two
coefficients or a transformation of one, with their standard errors by the delta method: each
definition is written as `name = expression` over the model's terms, `m.derive(ratio = educ /
exper)`, and inside the call a bare name is a term of the model, never a column or a binding.
The expression may use `+`, `-`, `*`, `/`, `^` and `log`, `exp`, `sqrt`, `log1p`, `expm1`,
`log10`, `log2` and `abs`; anything else is refused. Like Stata's `nlcom` after `regress` and
`car::deltaMethod` in R.

## Arguments
- fit: The regression fit.
- ...: The definitions, `name = expression` over the terms.

## Value
A `Table` with one row per definition in the coefficient table's columns: the estimate, its
standard error, the t value on the fit's degrees of freedom, the p-value and the interval at the
fit's level.

## Details
Each definition is evaluated at the estimates with its gradient over the coefficients by
forward-mode dual numbers; the standard error is √(gᵀ V g) with V the fit's variance matrix,
so robust or clustered errors carry through. The t statistic and the interval use the fit's
residual degrees of freedom, as Stata's `nlcom` does after `regress`.

## References
Oehlert, G. W. (1992). A note on the delta method. *The American Statistician*, 46, 27–29.
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapter 5.

## See also
test, compare, coef, vcov

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.derive(ratio = educ / exper, gain = exp(educ * 4) - 1)
derive · m

  term    estimate   std.error   t value   p value   95% CI
  ratio     3.4518      0.0775     44.55    <0.001   [3.2999, 3.6037]
  gain      0.3553      0.0074     48.02    <0.001   [0.3408, 0.3698]
