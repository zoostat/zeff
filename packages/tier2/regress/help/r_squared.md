# r_squared

## Summary
R squared and adjusted R squared

## Description
Gives the coefficient of determination of a regression and its adjusted form as a record, so
that `m.r_squared().adjusted` is one step away; both are `NA` when the outcome has no variation.
For a fit with fixed effects, `r_squared` and `adjusted` are the overall forms, with the effects'
fit included, and the record gains `within` and `adjusted_within`, the forms of the regression
on the demeaned columns, as fixest's `r2`, `ar2`, `wr2` and `war2`. Like `summary(fit)$r.squared`
in R, `fixest::r2`, and `e(r2)` and `e(r2_within)` in Stata.

## Arguments
- fit: The regression fit.

## Value
A `Record` with the fields `r_squared` and `adjusted`, and with fixed effects `within` and
`adjusted_within` too.

## Details
R² is 1 − RSS/TSS, the residual sum of squares over the total sum of squares about the
(weighted) mean of the outcome with an intercept and about zero without, as R computes them;
adjusted R² is 1 − (1 − R²)(N − 1)/(N − k) with an intercept and 1 − (1 − R²) N/(N − k) without,
with N the observations used and k the terms with an estimate. With fixed effects, R² overall
takes the total sum of squares about the outcome's mean and the residuals of the fit with the
effects, and k counts the effects' parameters (the levels of every effect, less one for each
effect after the first) in its adjusted form; R² within takes the demeaned outcome's sum of
squares, and its adjusted form is 1 − (1 − R²w)(N − p)/(N − k − p) with p the effects'
parameters, which is fixest's `war2`.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapter 3.

## See also
fit_measures, summary, influence

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.r_squared()
r_squared: 0.202792
adjusted: 0.20266
> m.r_squared().adjusted
0.20266
> let m2 = dataset(wages).reg(log(wage) ~ educ + exper + female, fe: industry + year)
> m2.r_squared()
r_squared: 0.225852
adjusted: 0.224785
within: 0.178307
adjusted_within: 0.178171
