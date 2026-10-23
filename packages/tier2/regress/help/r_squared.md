# r_squared

## Summary
R squared and adjusted R squared

## Description
Gives the coefficient of determination of a regression and its adjusted form as a record, so
that `m.r_squared().adjusted` is one step away; both are `NA` when the outcome has no variation.
With fixed effects (release 0.6.0) the record gains the within and overall forms, labelled. Like
`summary(fit)$r.squared` in R and `e(r2)` in Stata.

## Arguments
- fit: The regression fit.

## Value
A `Record` with the fields `r_squared` and `adjusted`.

## Details
R² is 1 − RSS/TSS, the residual sum of squares over the total sum of squares about the
(weighted) mean of the outcome with an intercept and about zero without, as R computes them;
adjusted R² is 1 − (1 − R²)(N − 1)/(N − k) with an intercept and 1 − (1 − R²) N/(N − k) without,
with N the observations used and k the terms with an estimate.

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
