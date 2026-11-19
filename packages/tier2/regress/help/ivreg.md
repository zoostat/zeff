# ivreg

## Summary
Two-stage least squares: a regression with instrumented regressors

## In brief
- `staff.ivreg(productivity ~ training, instruments: training ~ offer)`: the effect of `training`, with `offer` as its instrument.
- The instrumented regressor is written in the formula like any other; `instruments:` says what stands in for it.
- `fe:`, `se:`, `cluster:` and `weights:` work as in `reg`; `first: true` prints the first stage above the table.

## Description
Fits a linear model in which one or more regressors are instrumented: each is related to the
error of the equation, so least squares would not give its effect, and one or more instruments
stand in for it. An instrument is a variable that moves the regressor and has no other way to
the outcome. The estimator is two-stage least squares: each instrumented regressor is first
fitted on the instruments and the other regressors, and the outcome is then regressed on the
fitted values.

The formula is the equation of interest, written as for `reg`, the instrumented regressors
among the others. `instruments:` names them on the left of `~` and the instruments that are
not in the equation on the right, several of each joined by `+`. The other regressors of the
formula are instruments for themselves and are not repeated.

Under the table the result says what was instrumented by what and gives the F statistic that
shows whether the instruments are strong enough to trust the estimate. When they are not, a
warning says so (W1007).

## Arguments
- data: A frame.
- formula: The equation of interest, the instrumented regressors written like any others: `y ~ w + x`.
- fe: Fixed effects to absorb, as in `reg`: variables or interactions `a:b`, joined by `+`. They are taken out of the outcome, the regressors and the instruments alike.
- instruments: What is instrumented, and by what: `x ~ z`, or `x1 + x2 ~ z1 + z2 + z3`. Each term on the left is a regressor of the formula; each on the right is a variable outside it.
- se: The standard errors: `classical`, `robust` (HC1), `hc2` or `hc3`; `classical` unless given, and `robust` with sampling weights. With several effects in `fe:`, `hc2` and `hc3` are refused (Z0410); with one effect they are available.
- cluster: A variable whose values are the clusters; the standard errors are then clustered (CR1). A number, text, a category, a date or a time serves, as in `reg`.
- weights: A variable of weights, zero or positive.
- weight_type: What the weights are: `frequency`, `analytic` or `sampling`; `sampling` unless given.
- level: The confidence level of the intervals; 0.95 unless given.
- first: Whether the first stage of each instrumented regressor is printed above the table; `false` unless given.

## Value
An `IvFit`: the table of estimates with their standard errors, t values, p-values and
confidence intervals, and under it the instrumented regressors, the instruments used and the
weak-instrument F. It is read as any fit is: `coef`, `std_errors`, `p_values` and `ci` by term,
`summary` for the table, `test` for hypotheses about the coefficients. `predict`, `fitted` and
`residuals` are those of the equation as written, with the regressors as observed; `design`
gives the columns, the instruments among them; `report` sets the fit beside others.

## Missing values
A row with `NA` in the outcome, a regressor, an instrument, a fixed effect, the weights or the
cluster variable is left out, and the line under the title counts the rows dropped. Both stages
use the same rows.

## Details
### The two stages
The instruments are the regressors that are not instrumented, the intercept among them, and
the instruments named on the right of `~`. Each instrumented regressor is regressed on all of
them, and the outcome is regressed on the regressors with each instrumented one replaced by
its fitted values. The residuals, and so the standard errors, are those of the equation as
written, the outcome less the regressors as observed times the estimates, not those of the
second regression.

### What is instrumented
Only the terms named on the left of `~`. An interaction or a power of an instrumented variable
is another regressor: to instrument `x:w` as well, name it, with an instrument for it,
`instruments: x + x:w ~ z + z:w`. A term left out of `instruments:` is taken as given.

### Identification
The model needs at least as many instruments outside the equation as it has instrumented
regressors, and they must move those regressors apart from one another. An instrument that is
already a regressor of the formula counts for nothing and is named in a note. An instrument
that does not vary, or that is a combination of the regressors and the instruments before it,
is dropped with a note; so is one that the fixed effects absorb. If too few remain the model
is refused (Z0804).

### Standard errors
The variance is that of `reg`, with the fitted regressors in the place of the regressors and
the residuals of the equation as written: the classical one divides the residual sum of
squares by N − k, the robust one carries N / (N − k), the clustered one
G / (G − 1) · (N − 1) / (N − k), and the table is a t table. Fixed effects count in k as they
do for `reg`.

### How strong the instruments are
Under classical errors the line under the table gives the Cragg–Donald F; under robust or
clustered errors the Kleibergen–Paap F, and beside it, when one regressor is instrumented, the
effective F of Montiel Olea and Pflueger with its critical value. With one instrumented
regressor the first two are the F of the instruments in its first stage. A value under 10, or
an effective F under its critical value, raises the warning: the estimate then leans towards
the least-squares one, and its interval is too narrow.

### No R²
The table prints no R². With a regressor replaced by its fitted values the residuals are not
those of a projection, the figure can be negative, and it says nothing of how well the
instruments work.

### Exactness
Closed form in double precision, by QR factorisation of the instruments and of the fitted
regressors: the estimates and standard errors agree with R to a relative 1e-8 on the reference
models, and to 1e-7 with fixed effects, which are taken out by an iteration.

## A common mistake
Naming an instrumented variable in `instruments:` and leaving it out of the formula:

> dataset(staff).ivreg(productivity ~ age, instruments: training ~ offer)
error[Z0504]: `training` is instrumented but is not a regressor
  = help: add it to the regressors: `productivity ~ age + training`

The instrumented regressor belongs in the formula; `instruments:` only says how it is
instrumented.

## Coming from R or Stata
| d.ivreg(y ~ w + x, instruments: x ~ z) | m <- AER::ivreg(y ~ w + x, ~ w + z, data = d) | ivregress 2sls y w (x = z), small |
| d.ivreg(y ~ w + x, instruments: x ~ z, se: robust) | sandwich::vcovHC(m, type = "HC1") | ivregress 2sls y w (x = z), vce(robust) small |
| d.ivreg(y ~ w + x, instruments: x ~ z, cluster: g) | sandwich::vcovCL(m, cluster = ~g) | ivregress 2sls y w (x = z), vce(cluster g) small |
| d.ivreg(y ~ w + x, fe: f, instruments: x ~ z) | fixest::feols, with f and x ~ z as further parts of its formula | ivreghdfe y w (x = z), absorb(f) |

R and Stata put the instruments inside the formula, after a bar or between brackets, and R's
`ivreg` repeats there the regressors that are not instrumented; here the instruments have an
argument of their own, and the instrumented regressor stays in the formula. Stata's
`ivregress 2sls` without `small` divides by N where this function divides by N − k, and prints
z statistics: its standard errors are these times the square root of (N − k) / N. fixest
reports a Cragg–Donald statistic with other degrees of freedom; the one printed here is Stock
and Yogo's, the minimum eigenvalue statistic of Stata's `estat firststage`.

## Related tasks
- To fit the same equation without instruments, for comparison, use `reg`.
- To see the first stage of each instrumented regressor and the tests of the instruments, use `first_stage`.
- For a test and a confidence set that hold however weak the instruments are, use `ar_test`.
- To set two or more fits side by side in one table, use `report`.
- To predict the outcome for other rows from the estimated equation, use `predict`.
- To see the columns the fit was computed on, with the instruments, use `design`.
- To read one coefficient or its interval from the fit, use `coef` and `ci`.
- To test a hypothesis about the coefficients, use `test`.

## References
Wooldridge, J. M. (2010), Econometric Analysis of Cross Section and Panel Data, 2nd edition,
MIT Press, chapter 5.

Staiger, D. and Stock, J. H. (1997), Instrumental variables regression with weak instruments,
Econometrica 65(3), 557–586.

Cragg, J. G. and Donald, S. G. (1993), Testing identifiability and specification in
instrumental variable models, Econometric Theory 9(2), 222–240.

Stock, J. H. and Yogo, M. (2005), Testing for weak instruments in linear IV regression, in
Andrews, D. W. K. and Stock, J. H. (eds.), Identification and Inference for Econometric Models,
Cambridge University Press, 80–108.

Kleibergen, F. and Paap, R. (2006), Generalized reduced rank tests using the singular value
decomposition, Journal of Econometrics 133(1), 97–126.

Montiel Olea, J. L. and Pflueger, C. (2013), A robust test for weak instruments, Journal of
Business and Economic Statistics 31(3), 358–369.

## See also
reg, first_stage, ar_test, report, coef, ci, test, predict, design

## Examples
// The effect of a training course on productivity: the offer of the course was drawn by lot, so it is an instrument for taking it.
> let staff = dataset(staff)
> staff.ivreg(productivity ~ training, instruments: training ~ offer)
ivreg · productivity ~ training
N = 2,400
standard errors: classical

  term          estimate   std.error   t value   p value   95% CI
  (intercept)    51.0507      0.2743    186.10    <0.001   [50.5127, 51.5886]
  training        4.3177      0.4975      8.68    <0.001   [3.3420, 5.2933]

instrumented: training · instruments: offer
Cragg–Donald F 740.58

// Least squares on the same equation overstates the effect: the able take the course more often.
> staff.reg(productivity ~ training)
reg · productivity ~ training
N = 2,400
standard errors: classical

  term          estimate   std.error   t value   p value   95% CI
  (intercept)    50.4989      0.1693    298.32    <0.001   [50.1670, 50.8309]
  training        5.4323      0.2406     22.58    <0.001   [4.9605, 5.9041]

R² 0.175 · adjusted R² 0.175

// The sites absorbed, a second instrument, robust errors, and the first stage printed above.
> staff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, se: robust, first: true)
first stage · training ~ age + offer + distance · fe: site

  term       estimate   std.error   t value   p value   95% CI
  age         -0.0039      0.0008     -4.87    <0.001   [-0.0054, -0.0023]
  offer        0.4841      0.0176     27.50    <0.001   [0.4496, 0.5187]
  distance    -0.0059      0.0015     -3.95    <0.001   [-0.0088, -0.0029]

F of the excluded instruments 396.58 on 2 and 2,385 degrees of freedom (robust)

ivreg · productivity ~ age + training · fe: site
N = 2,400 (no singletons)
fixed effects: site (12)
standard errors: robust (HC1)

  term       estimate   std.error   t value   p value   95% CI
  age         -0.0005      0.0095     -0.05     0.959   [-0.0190, 0.0181]
  training     4.5816      0.4231     10.83    <0.001   [3.7519, 5.4113]

instrumented: training · instruments: offer, distance
Kleibergen–Paap F 396.58 · effective F 383.11 (critical value 19.66)

// Standard errors clustered by site; twelve clusters are few, and the warning says so.
> staff.ivreg(productivity ~ age + training, instruments: training ~ offer, cluster: site)
ivreg · productivity ~ age + training
N = 2,400
standard errors: clustered by site (12 clusters, CR1)

  term          estimate   std.error   t value   p value   95% CI
  (intercept)    51.0791      0.8987     56.84    <0.001   [49.1011, 53.0571]
  age            -0.0007      0.0073     -0.10     0.925   [-0.0167, 0.0153]
  training        4.3185      0.4085     10.57    <0.001   [3.4193, 5.2177]

instrumented: training · instruments: offer
Kleibergen–Paap F 630.53 · effective F 630.53 (critical value 23.11)
warning[W1002]: few clusters: 12 clusters give clustered errors little to rest on (line 5)
  = help: the wild cluster bootstrap (`se: bootstrap`, release 1.1) is the usual remedy below 30 clusters

// Weights and their kind, and 99% intervals.
> staff.ivreg(productivity ~ training, instruments: training ~ offer, weights: tenure, weight_type: analytic, level: 0.99)
ivreg · productivity ~ training
N = 2,400
weights: tenure (analytic)
standard errors: classical

  term          estimate   std.error   t value   p value   99% CI
  (intercept)    51.3391      0.3105    165.36    <0.001   [50.5387, 52.1394]
  training        4.3191      0.5076      8.51    <0.001   [3.0106, 5.6275]

instrumented: training · instruments: offer
Cragg–Donald F 731.67

// A weak instrument: the distance alone moves the take-up little, and the warning follows.
> staff.ivreg(productivity ~ training, instruments: training ~ distance, se: robust)
ivreg · productivity ~ training
N = 2,400
standard errors: robust (HC1)

  term          estimate   std.error   t value   p value   95% CI
  (intercept)    52.2978      1.7541     29.82    <0.001   [48.8582, 55.7375]
  training        1.7981      3.5424      0.51     0.612   [-5.1484, 8.7446]

instrumented: training · instruments: distance
Kleibergen–Paap F 12.05 · effective F 12.05 (critical value 23.11)
warning[W1007]: weak instruments: the effective F is 12.05, under its critical value of 23.11 (line 7)
  = help: with weak instruments the estimate leans towards the least-squares one and its confidence interval is too narrow; `ar_test()` gives a test and a confidence set that hold however weak the instruments are

// Keep the fit under a name to read its parts.
> let iv = staff.ivreg(productivity ~ training, fe: site, instruments: training ~ offer)

// The estimate of the course's effect, whose true value in these data is 4.5.
> iv.coef(training)
4.60265

// And its confidence interval.
> iv.ci(training)
3.76049
5.44481

// Beside least squares in one table: the row of instruments marks the instrumented model, which has no R².
> report([staff.reg(productivity ~ training, fe: site), iv])
                           (1)         (2)
  training              5.926***    4.603***
                       (0.207)     (0.429)
  ──────────────────────────────────────────
  Fixed effects             site        site
  Instruments                          offer
  SEs                  classical   classical
  N                        2,400       2,400
  R²              0.256 (within)

* p < 0.05, ** p < 0.01, *** p < 0.001
