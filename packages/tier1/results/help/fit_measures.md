# fit_measures

## Summary
Fit statistics of a model

## Description
Gives the fit measures of a fitted model as a record: the Gaussian log-likelihood, AIC and BIC
(which count the variance as a parameter, as R's `AIC` and `BIC` do), R² and adjusted R² (`NA`
when the outcome has no variation), so that a figure is one `.name` away. A test's result has
none and gives `NA`. Like `logLik`, `AIC` and `BIC` in R and `estat ic` in Stata.

## Arguments
- fit: The result.

## Value
A `Record` with the fields `log_likelihood`, `aic`, `bic`, `r_squared` and `adjusted`.

## Details
The log-likelihood is the Gaussian one at the least-squares estimates, −(N/2)(log 2π + 1 + log(RSS/N)),
with N the observations used; AIC is −2ℓ + 2(k + 1) and BIC is −2ℓ + ln(N)(k + 1), with k the
terms with an estimate and one more for the variance, as R's `AIC` and `BIC` count them. R² and
adjusted R² are those of the summary.

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
