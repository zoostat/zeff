# glm

## Summary
A generalized linear model: logistic and probit regression of a binary outcome

## In brief
- `wages.glm(union ~ educ + exper, family: binomial)`: a logistic regression of union membership.
- `family:` is required; `link: probit` gives a probit regression in the place of the logit.
- `se: robust` gives robust standard errors and `cluster: worker` errors clustered by worker.
- `weights: w` weights the rows: survey weights unless `weight_type: frequency` says counts.
- The table gives z values, as maximum likelihood does; under it, the log-likelihood and the deviance.

## Description
Fits a model of an outcome that is 0 or 1 (or false or true) by maximum likelihood. The
probability that the outcome is 1 is a function of a linear combination of the regressors:
under the logit link, the log of the odds is that combination; under the probit link, it is the
point of the standard normal distribution below which the probability lies.

The formula is written as for `reg`. `family: binomial` says that the outcome is a binary one
and must be written: a model left without a family is not taken for a linear one, which `reg`
fits. Further families and links are added from release 1.7.

The estimates are on the scale of the link: under the logit link a coefficient is the change in
the log of the odds for one unit of its regressor, and its exponential the ratio of the odds.

## Arguments
- data: A frame.
- formula: The model: the outcome, 0 and 1 or a Bool, on the left of `~`, the regressors on the right, written as for `reg`.
- family: The family of the outcome's distribution: `binomial`, for an outcome of 0 and 1. Required.
- link: The link: `logit` (logistic regression) or `probit` (probit regression); `logit` unless given.
- fe: Fixed effects to absorb, as `reg` takes them; on `glm` from release 1.7, and refused until then (Z0304). A category among the regressors does the same for a few groups.
- se: The kind of standard error: `classical`, `robust` (HC1), `hc2` or `hc3`; `classical` unless sampling weights are given, when `robust`.
- cluster: The variable whose values group the observations for clustered errors (CR1): a number, text, a category, a date or a time, each cluster the rows that share a value.
- weights: The weight variable.
- weight_type: What the weights are: `frequency` counts or `sampling` probability weights; `sampling` unless given. `analytic` precision weights are refused (Z0410): a binary outcome has no error variance for them to scale.
- level: The confidence level of the intervals; 0.95 unless given.

## Value
A `GlmFit`: the table of estimates with their standard errors, z values, p-values and
confidence intervals, and under it the log-likelihood, AIC, McFadden's R², the deviance and the
null deviance, and the iterations the fit took. It is read as any fit is: `coef`, `std_errors`,
`p_values`, `ci` and `estimates` by term, `vcov` for the variance of the estimates, `summary`
for the table; `fit_measures` for the log-likelihood, AIC, BIC and the deviances as numbers;
`fitted` for the fitted probabilities, `residuals` for the residuals of each kind, and `predict`
for the probabilities on new data.

## Missing values
A row with `NA` in the outcome, a regressor, the weight or the cluster variable is left out, and
the line under the title counts the rows dropped. A row with a zero weight is left out too,
with a note.

## Details
### The fit
The maximum of the likelihood is found by Fisher scoring, iteratively reweighted least squares,
from R's start, each row's mean moved halfway towards one half. A step that lowers the
likelihood is halved. The iterations stop when the log-likelihood and the estimates have both
stopped changing, at a relative 1e-13 and 1e-11; the count is printed under the table. Where
regressors are nearly collinear the estimates cannot settle so far, and the iterations stop
when the fitted values no longer change beyond rounding: the fit has then converged to the
precision the design allows.

### Standard errors
The classical variance is the inverse of the expected information at the maximum. The table is
a z table: the statistic is the estimate over its standard error, and the p-value and the
interval are those of the standard normal distribution, whatever the kind of error.

`se: robust` is the sandwich of the observations' scores between the inverse information, times
N/(N − k), HC1; `hc2` and `hc3` scale each observation's score by 1/(1 − h) and 1/(1 − h)², with
h its leverage in the weighted fit. `cluster:` sums the scores within each cluster first, times
G/(G − 1) · (N − 1)/(N − k) for G clusters, CR1: the factors `reg` uses, since every model takes
its errors from one computation. `hc2` or `hc3` with `cluster:` is refused (Z0410), and so is
one cluster (Z0802); fewer than 30 clusters are warned of (W1002). A row with `NA` in the
cluster variable is left out.

### Weights
Weights multiply each row's log-likelihood. With `weight_type: frequency` a row stands for as
many identical observations as its weight, a whole number: the fit is the one on the rows
written out, N is the sum of the weights and the classical errors hold. With `weight_type:
sampling`, the default, the weights are probability weights, as a survey gives: what is
maximised is a pseudo-likelihood, so the line under the table says log pseudo-likelihood, which
compares no models; the errors are robust unless `cluster:` is given, and classical errors are
refused (Z0410). Sampling weights are scaled to sum to the rows used, which changes neither the
estimates nor their errors. Analytic weights, which say how precisely each outcome was measured,
are refused (Z0410): the variance of a 0 or 1 is fixed by its mean. A negative weight is an
error (Z0224); a few rows carrying most of the weight are warned of (W1003).

### Collinearity
A regressor that is a combination of the regressors before it is dropped with a note, as in
`reg`, and so is a column that is zero in every row used. Regressors that are nearly, not
exactly, a combination are kept, and the fit is warned of when the condition number of the
scaled, weighted design passes 1e8 (W1001): their estimates are then large, of opposite signs
and poorly determined, though the fitted probabilities and the log-likelihood are not.

### When there is no estimate
The maximum of the likelihood does not always exist, and the fit then stops with an error
rather than print estimates that mean nothing. An outcome that is 0 in every row used, or 1 in
every row, has its probability estimated at 0 or 1 whatever the regressors (Z0803); so does a
regressor that separates the 0s from the 1s, whose estimate runs off to infinity (Z0803). The
separation is found before the first iteration, by a linear program (Konis's), and the message
names the regressor that separates the outcome alone, or the regressors that do it together; a
level of a category with only 0s or only 1s is named as its column, `industry=Mining`. A
model with as many regressors as rows, or more, is refused as `reg` refuses it (Z0802). A fit
that has not converged in 100 iterations is refused with the change of the log-likelihood at the
last (Z0803), and so, rarely, is one in which the weighted design can no longer tell a
regressor from the others, or whose log-likelihood is not a finite number.

### Probabilities at 0 or 1
A fit may converge and still put a row's probability at 0 or 1 to the machine's precision: an
observation far out on a regressor, on the side its estimate points to. The outcomes are not
separated, so the estimates exist, but they lean on the rows near that observation; the fit is
printed with a warning that counts such rows (W1008), by R's rule, a probability within ten
machine epsilons of 0 or 1.

### Exactness
Iterative: the estimates, the standard errors and the log-likelihood agree with R's `glm`,
iterated to its maximum, to a relative 1e-9 on the reference models. R's own rule stops its
iterations on the change of the deviance alone and may stop a probit fit some 1e-10 short of
the maximum.

## A common mistake
Leaving out the family:

> dataset(wages).glm(union ~ educ)
error[Z0406]: `glm` needs `family:`
  = help: a binary outcome is `family: binomial`; a linear model is `reg`

`family:` has no default, so that a binary model is never fitted by mistake as another.

## Coming from R or Stata
| d.glm(y ~ x, family: binomial) | glm(y ~ x, family = binomial, data = d) | logit y x |
| d.glm(y ~ x, family: binomial, link: probit) | glm(y ~ x, family = binomial(link = "probit"), data = d) | probit y x |

| d.glm(y ~ x, family: binomial, se: robust) | sandwich::vcovHC(m, type = "HC1") | logit y x, vce(robust) |
| d.glm(y ~ x, family: binomial, cluster: id) | sandwich::vcovCL(m, cluster = ~id, type = "HC1") | logit y x, vce(cluster id) |
| d.glm(y ~ x, family: binomial, weights: w) | glm(y ~ x, family = quasibinomial, data = d, weights = w) with sandwich::vcovHC(m, type = "HC1") | logit y x [pweight = w] |
| d.glm(y ~ x, family: binomial, weights: n, weight_type: frequency) | glm(y ~ x, family = binomial, data = d, weights = n) | logit y x [fweight = n] |

R writes the family as a function, `binomial(link = "probit")`; here the family and the link are
two words. In R the robust and clustered errors come from the package sandwich, and are these
when its `type` is "HC1"; left to itself, `vcovCL` gives a `glm` fit G/(G − 1) alone, which is
also Stata's factor after `vce(cluster id)`, so those errors are smaller by the factor
√((N − k)/(N − 1)), a hair in any sample of size. Stata's `vce(robust)` uses N/(N − 1) where
HC1 has N/(N − k). R's `weights =` are counts of trials, the frequency weights here; for
survey weights R's package survey fits the same estimates. Stata's `logit` takes no analytic
weights either. Stata's `logit` and `probit` give the same estimates; its `probit` takes its standard
errors from the observed information, where R and this function take the expected, so the two
differ slightly; under the logit link they are one.

## Related tasks
- To fit a linear probability model of the same outcome, use `reg`.
- To read one coefficient or its interval from the fit, use `coef` and `ci`.
- To allow for observations of one worker being alike, add `cluster: worker`.
- To get each row's fitted probability, use `fitted`; for new rows, `predict`.
- To test whether terms can be left out, fit both models and use `compare`; for one restriction, `test`.
- To get an odds ratio with its standard error, use `derive`, with the exponential of the term.
- To see every estimate with its statistic as a frame, use `estimates`.

## References
McCullagh, P. and Nelder, J. A. (1989), Generalized Linear Models, 2nd edition, Chapman and
Hall, chapters 2 and 4.

Agresti, A. (2013), Categorical Data Analysis, 3rd edition, Wiley, chapter 5.

## See also
reg, coef, ci, estimates, vcov, fit_measures, predict, residuals, test, compare, derive

## Examples
// A logistic regression of union membership on schooling, experience and sex.
> let wages = dataset(wages)
> wages.glm(union ~ educ + exper + female, family: binomial)
glm · union ~ educ + exper + female · binomial, logit
N = 18,390 (22 rows dropped for NA in model variables)
standard errors: classical

  term          estimate   std.error   z value   p value   95% CI
  (intercept)    -1.2274      0.1100    -11.16    <0.001   [-1.4429, -1.0119]
  educ            0.0009      0.0068      0.14     0.892   [-0.0123, 0.0142]
  exper           0.0277      0.0022     12.32    <0.001   [0.0233, 0.0321]
  female          0.0556      0.0318      1.75     0.081   [-0.0068, 0.1180]

log-likelihood -11,415.06 · AIC 22,838.13 · McFadden R² 0.007
deviance 22,830.13 on 18,386 df · null deviance 23,001.97 on 18,389 df
converged in 5 iterations of Fisher scoring

// The probit link, with 99% intervals.
> wages.glm(union ~ educ + exper + female, family: binomial, link: probit, level: 0.99)
glm · union ~ educ + exper + female · binomial, probit
N = 18,390 (22 rows dropped for NA in model variables)
standard errors: classical

  term          estimate   std.error   z value   p value   99% CI
  (intercept)    -0.7532      0.0666    -11.30    <0.001   [-0.9248, -0.5816]
  educ            0.0005      0.0041      0.13     0.894   [-0.0100, 0.0111]
  exper           0.0167      0.0014     12.29    <0.001   [0.0132, 0.0203]
  female          0.0342      0.0193      1.77     0.077   [-0.0156, 0.0840]

log-likelihood -11,415.43 · AIC 22,838.86 · McFadden R² 0.007
deviance 22,830.86 on 18,386 df · null deviance 23,001.97 on 18,389 df
converged in 8 iterations of Fisher scoring

// Robust standard errors (HC1).
> wages.glm(union ~ educ + exper + female, family: binomial, se: robust)
glm · union ~ educ + exper + female · binomial, logit
N = 18,390 (22 rows dropped for NA in model variables)
standard errors: robust (HC1)

  term          estimate   std.error   z value   p value   95% CI
  (intercept)    -1.2274      0.1092    -11.24    <0.001   [-1.4415, -1.0133]
  educ            0.0009      0.0067      0.14     0.891   [-0.0122, 0.0141]
  exper           0.0277      0.0023     12.17    <0.001   [0.0232, 0.0322]
  female          0.0556      0.0318      1.75     0.081   [-0.0068, 0.1180]

log-likelihood -11,415.06 · AIC 22,838.13 · McFadden R² 0.007
deviance 22,830.13 on 18,386 df · null deviance 23,001.97 on 18,389 df
converged in 5 iterations of Fisher scoring

// Errors clustered by worker: the same estimates, standard errors more than twice as large.
> wages.glm(union ~ educ + exper + female, family: binomial, cluster: worker)
glm · union ~ educ + exper + female · binomial, logit
N = 18,390 (22 rows dropped for NA in model variables)
standard errors: clustered by worker (3,911 clusters, CR1)

  term          estimate   std.error   z value   p value   95% CI
  (intercept)    -1.2274      0.2654     -4.63    <0.001   [-1.7475, -0.7073]
  educ            0.0009      0.0165      0.06     0.956   [-0.0315, 0.0333]
  exper           0.0277      0.0054      5.13    <0.001   [0.0171, 0.0383]
  female          0.0556      0.0796      0.70     0.485   [-0.1005, 0.2116]

log-likelihood -11,415.06 · AIC 22,838.13 · McFadden R² 0.007
deviance 22,830.13 on 18,386 df · null deviance 23,001.97 on 18,389 df
converged in 5 iterations of Fisher scoring
note: 3 of the 3,911 clusters hold one observation (line 5)

// Survey weights: the hours worked as sampling weights, with the robust errors they bring.
> wages.glm(union ~ educ + exper + female, family: binomial, weights: hours)
glm · union ~ educ + exper + female · binomial, logit
N = 18,355 (57 rows dropped for NA in model variables)
weights: hours (sampling)
standard errors: robust (HC1)

  term          estimate   std.error   z value   p value   95% CI
  (intercept)    -1.1947      0.1117    -10.70    <0.001   [-1.4136, -0.9758]
  educ            0.0010      0.0069      0.14     0.885   [-0.0125, 0.0145]
  exper           0.0273      0.0023     11.73    <0.001   [0.0227, 0.0318]
  female          0.0597      0.0326      1.83     0.067   [-0.0041, 0.1235]

log pseudo-likelihood -11,478.76 · AIC 22,965.51 · McFadden R² 0.007
deviance 22,957.51 on 18,351 df · null deviance 23,126.63 on 18,354 df
converged in 6 iterations of Fisher scoring

// Frequency weights: each woman's row counted twice.
> wages.add(copies = 1 + female).glm(union ~ educ + exper, family: binomial, weights: copies, weight_type: frequency)
glm · union ~ educ + exper · binomial, logit
N = 27,303 (the sum of the frequency weights over 18,390 rows; 22 rows dropped for NA in model variables)
weights: copies (frequency)
standard errors: classical

  term          estimate   std.error   z value   p value   95% CI
  (intercept)    -1.0523      0.0889    -11.84    <0.001   [-1.2265, -0.8782]
  educ           -0.0072      0.0055     -1.30     0.192   [-0.0181, 0.0036]
  exper           0.0258      0.0018     14.04    <0.001   [0.0222, 0.0294]

log-likelihood -16,989.73 · AIC 33,985.46 · McFadden R² 0.007
deviance 33,979.46 on 27,300 df · null deviance 34,217.31 on 27,302 df
converged in 6 iterations of Fisher scoring
