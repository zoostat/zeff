# vif

## Summary
Variance inflation factors for the regressors

## Description
How much each coefficient's variance is inflated by the correlation of its regressor with the
others: for regressor j, 1 / (1 − R²ⱼ), where R²ⱼ is the R² of regressing that column of the
design on the other columns and the intercept, over the rows the fit used. A factor of 1 means a
regressor orthogonal to the rest; factors above 10 are the usual sign of collinearity worth
looking into, though no threshold is a rule. The intercept has no factor, as `car::vif` leaves
it out; each column of a categorical regressor has its own row under its term name (section 7.2
of the Language Reference), the generalised factor of a whole categorical term being a later
addition. A fit without an intercept gives the uncentred factors, as Stata's `estat vif` does
after `noconstant`. Rescaling a regressor changes no factor. Like `car::vif` for single-column
terms and Stata's `estat vif`.

## Arguments
- fit: A regression fit.

## Value
A table titled `vif · fit` with the columns `term` and `vif`, to three decimals; a note under it
names the terms dropped as collinear, which have no factor.

## Details
From the Gram matrix G = XᵀX of the fitted design (the weighted design under weights): the factor of column j is (G⁻¹)ⱼⱼ times its centred sum of squares, Gⱼⱼ − G₀ⱼ² / G₀₀ with column 0 the intercept, which is 1 / (1 − R²ⱼ); for a fit without an intercept the plain Gⱼⱼ, the uncentred form. The Gram matrix comes from the scaled R factor of the fit's QR decomposition, so no second pass over the data is made.

## References
Fox, J. and Monette, G. (1992) Generalized collinearity diagnostics, Journal of the American Statistical Association 87(417), 178–183.
Belsley, D. A., Kuh, E. and Welsch, R. E. (1980) Regression Diagnostics, Wiley, chapter 3.

## See also
reg, r_squared, influence, summary

## Examples
> let w = dataset(wages)
> let m = w.reg(log(wage) ~ educ + exper + exper^2 + female)
> m.vif()
vif · m

  term         vif
  educ       1.118
  exper     14.101
  exper^2   14.008
  female     1.001
