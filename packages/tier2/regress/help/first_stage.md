# first_stage

## Summary
The first stage of a fit by instrumental variables, with its diagnostics

## In brief
- `iv.first_stage()`: each instrumented regressor on the instruments, as a table, with its partial R² and F.
- Under the tables: how strong the instruments are, whether the extra instruments agree, and whether instrumenting was needed.
- Every figure stands on the standard errors the fit was given.

## Description
Shows what the instruments do in a model fitted by `ivreg`. For each instrumented regressor it
prints the first stage: the regression of that regressor on all the instruments, those
outside the equation and the regressors that are not instrumented. Under each table are the
partial R², the share of the regressor that the outside instruments explain once the others
are accounted for, and the F statistic of those instruments.

Three lines for the model as a whole follow. The first gives the weak-instrument statistic
printed under the fit. The second tests the over-identifying restrictions: with more outside
instruments than instrumented regressors, whether the instruments agree with one another
about the estimate. The third is the Durbin–Wu–Hausman test: whether the regressors that were
instrumented differ enough from exogenous ones for instrumenting to have mattered.

## Arguments
- iv: A fit by `ivreg`.

## Value
A `Table`: one table for each instrumented regressor, then the lines for the model. It
prints, and it can be written to a file with `export`.

## Missing values
The first stage uses the rows the fit used: a row with `NA` in any variable of the model, the
instruments among them, was left out when the model was fitted.

## Details
### The tables
Each first stage is least squares of an instrumented regressor on the instruments the fit
kept, with the standard errors the fit was given: classical, robust or clustered, and with
its weights and fixed effects. An instrument dropped when the model was fitted is not in the
table.

### Partial R² and Shea's
The partial R² of a first stage takes the other instruments out of the regressor and out of
the instruments outside the equation, and gives the share of what is left of the regressor
that those instruments explain. With several instrumented regressors Shea's partial R² is
printed beside it: it also takes the other instrumented regressors out, so it is low when the
instruments move the regressors only together, even if each partial R² is high.

### The F of the excluded instruments
Under classical errors it is the F test that the coefficients of the instruments outside the
equation are all zero. Under robust or clustered errors it is the Wald statistic of the same
hypothesis on that variance, divided by the number of instruments, and the line says so.

### Weak instruments
The line repeats the statistic under the fit: the Cragg–Donald F under classical errors; the
Kleibergen–Paap F under robust or clustered ones, with the effective F of Montiel Olea and
Pflueger and its critical value when one regressor is instrumented.

### Over-identification
With as many outside instruments as instrumented regressors the model is exactly identified
and there is nothing to test, which the line says. With more, the test is Sargan's under
classical errors, N times the share of the residuals that the instruments explain, and
Hansen's J under robust or clustered errors. Both are chi-squared on the number of extra
instruments. A small p-value says that some instrument is related to the error, without
saying which; a large one does not prove that all are valid.

### Endogeneity
The Durbin–Wu–Hausman test adds the residuals of each first stage to the equation and tests
their coefficients: an F under classical errors, the Wald form on the robust or clustered
variance otherwise. A small p-value says the least-squares estimate and the instrumented one
differ by more than chance.

### Exactness
Closed form in double precision: the figures agree with R to a relative 1e-7 on the reference
models.

## A common mistake
Asking for the first stage of a model fitted by `reg`, which has none:

> dataset(staff).reg(productivity ~ training).first_stage()
error[Z0401]: `first_stage` cannot be called on a RegFit
  = help: `first_stage` takes an IvFit first; `x.first_stage(...)` is `first_stage(x, ...)`

Fit the model with `ivreg` and its `instruments:` first.

## Coming from R or Stata
| iv.first_stage() | summary(m, diagnostics = TRUE) for AER::ivreg; fixest::fitstat(m, ~ ivf + sargan + wh) | estat firststage, then estat overid and estat endogenous |
| the first-stage table alone | summary(m, stage = 1) in fixest | ivregress 2sls with the option first |

Stata's `estat firststage` prints Shea's partial R² and the minimum eigenvalue statistic, which
is the Cragg–Donald F printed here. After robust errors Stata's `estat overid` and
`estat endogenous` report score tests of the same two hypotheses; Hansen's J and the Wald form
of the Durbin–Wu–Hausman test printed here are those of `ivreg2`, and the figures are close
to Stata's without being equal. fixest's Wu–Hausman and Sargan statistics are the classical
ones printed here.

## Related tasks
- To fit the model whose first stage this is, use `ivreg`.
- When the instruments are weak, for a test and a confidence set that still hold, use `ar_test`.
- To set the instrumented fit beside a least-squares one, use `report`.
- To see the instruments and the regressors row by row, use `design`.
- To write the tables to a file, use `export`.

## References
Shea, J. (1997), Instrument relevance in multivariate linear models: a simple measure, Review
of Economics and Statistics 79(2), 348–352.

Sargan, J. D. (1958), The estimation of economic relationships using instrumental variables,
Econometrica 26(3), 393–415.

Hansen, L. P. (1982), Large sample properties of generalized method of moments estimators,
Econometrica 50(4), 1029–1054.

Hausman, J. A. (1978), Specification tests in econometrics, Econometrica 46(6), 1251–1271.

Stock, J. H. and Yogo, M. (2005), Testing for weak instruments in linear IV regression, in
Andrews, D. W. K. and Stock, J. H. (eds.), Identification and Inference for Econometric Models,
Cambridge University Press, 80–108.

Kleibergen, F. and Paap, R. (2006), Generalized reduced rank tests using the singular value
decomposition, Journal of Econometrics 133(1), 97–126.

Montiel Olea, J. L. and Pflueger, C. (2013), A robust test for weak instruments, Journal of
Business and Economic Statistics 31(3), 358–369.

## See also
ivreg, ar_test, report, design, export

## Examples
// One regressor instrumented by two instruments, with robust errors: the first stage and the three lines for the model.
> let staff = dataset(staff)
> let iv = staff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, se: robust)
> iv.first_stage()
first stage · training ~ age + offer + distance · fe: site

  term       estimate   std.error   t value   p value   95% CI
  age         -0.0039      0.0008     -4.87    <0.001   [-0.0054, -0.0023]
  offer        0.4841      0.0176     27.50    <0.001   [0.4496, 0.5187]
  distance    -0.0059      0.0015     -3.95    <0.001   [-0.0088, -0.0029]

partial R² 0.2446
F of the excluded instruments 396.58 on 2 and 2,385 degrees of freedom (robust)

standard errors: robust (HC1)
weak instruments: Kleibergen–Paap F 396.58 · effective F 383.11 (critical value 19.66)
over-identification: Hansen's J, chi2(1) = 0.13   p = 0.723 (robust)
endogeneity: Durbin–Wu–Hausman F(1, 2,385) = 13.74   p < 0.001 (robust)

// One instrument for one regressor: exactly identified, so there is no over-identification test.
> staff.ivreg(productivity ~ training, instruments: training ~ offer).first_stage()
first stage · training ~ offer

  term          estimate   std.error   t value   p value   95% CI
  (intercept)     0.2563      0.0125     20.49    <0.001   [0.2318, 0.2809]
  offer           0.4858      0.0179     27.21    <0.001   [0.4508, 0.5208]

partial R² 0.2360
F of the excluded instruments 740.58 on 1 and 2,398 degrees of freedom

standard errors: classical
weak instruments: Cragg–Donald F 740.58
over-identification: exactly identified, no over-identification test
endogeneity: Durbin–Wu–Hausman F(1, 2,397) = 6.64   p = 0.010
