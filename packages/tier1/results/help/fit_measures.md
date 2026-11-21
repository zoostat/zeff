# fit_measures

## Summary
Fit statistics of a model

## Description
Gives the fit measures of a fitted model as a record: the Gaussian log-likelihood, AIC and BIC
(which count the variance as a parameter, as R's `AIC` and `BIC` do), R² and adjusted R² (`NA`
when the outcome has no variation), so that a figure is one `.name` away. A test's result has
none and gives `NA`. Like `logLik`, `AIC` and `BIC` in R and `estat ic` in Stata.

For a `glm` fit the log-likelihood is the model's own, AIC and BIC count the coefficients alone,
and the record gives the deviance, the null deviance and McFadden's R² in the place of R² and
adjusted R²: the measures are those the model has.

## Arguments
- fit: The result.

## Value
A `Record`. For a regression, the fields `log_likelihood`, `aic`, `bic`, `r_squared` and
`adjusted`; for a `glm` fit, `log_likelihood`, `aic`, `bic`, `deviance`, `null_deviance` and
`r_squared_mcfadden`.

## Details
The log-likelihood is the Gaussian one at the least-squares estimates, −(N/2)(log 2π + 1 + log(RSS/N)),
with N the observations used; AIC is −2ℓ + 2(k + 1) and BIC is −2ℓ + ln(N)(k + 1), with k the
terms with an estimate and one more for the variance, as R's `AIC` and `BIC` count them. R² and
adjusted R² are those of the summary.

For a fit by maximum likelihood, AIC is −2ℓ + 2k and BIC is −2ℓ + ln(N)k, with N the
observations (the sum of the frequency weights, when there are any). The deviance is twice the
gap between the log-likelihood of the saturated model and the fit's, and the null deviance that
of the model with the intercept alone; McFadden's R² is one less the ratio of the fit's
log-likelihood to the null model's. Under sampling weights the likelihood is a
pseudo-likelihood, and these measures compare no models.

## References
Akaike, H. (1974). A new look at the statistical model identification. *IEEE Transactions on
Automatic Control*, 19, 716–723. Schwarz, G. (1978). Estimating the dimension of a model. *Annals
of Statistics*, 6, 461–464.

## See also
summary, r_squared, estimates

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.fit_measures()
log_likelihood: -10563.8
aic: 21137.5
bic: 21176.5
r_squared: 0.202792
adjusted: 0.20266
> m.fit_measures().aic
21137.5
> let g = dataset(wages).glm(union ~ educ + exper + female, family: binomial)
> g.fit_measures()
log_likelihood: -11415.1
aic: 22838.1
bic: 22869.4
deviance: 22830.1
null_deviance: 23002
r_squared_mcfadden: 0.00747066
