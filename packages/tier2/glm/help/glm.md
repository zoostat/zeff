# glm

## Summary
A generalized linear model: logistic and probit regression of a binary outcome

## In brief
- `wages.glm(union ~ educ + exper, family: binomial)`: a logistic regression of union membership.
- `family:` is required; `link: probit` gives a probit regression in the place of the logit.
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
- level: The confidence level of the intervals; 0.95 unless given.

## Value
A `GlmFit`: the table of estimates with their standard errors, z values, p-values and
confidence intervals, and under it the log-likelihood, AIC, McFadden's R², the deviance and the
null deviance, and the iterations the fit took. It is read as any fit is: `coef`, `std_errors`,
`p_values`, `ci` and `estimates` by term, `vcov` for the variance of the estimates, `summary`
for the table.

## Missing values
A row with `NA` in the outcome or a regressor is left out, and the line under the title counts
the rows dropped.

## Details
### The fit
The maximum of the likelihood is found by Fisher scoring, iteratively reweighted least squares,
from R's start, each row's mean moved halfway towards one half. A step that lowers the
likelihood is halved. The iterations stop when the log-likelihood and the estimates have both
stopped changing, at a relative 1e-13 and 1e-11; the count is printed under the table.

### Standard errors
The classical variance is the inverse of the expected information at the maximum. The table is
a z table: the statistic is the estimate over its standard error, and the p-value and the
interval are those of the standard normal distribution.

### Collinearity
A regressor that is a combination of the regressors before it is dropped with a note, as in
`reg`, and so is a column that is zero in every row used.

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
last (Z0803).

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

R writes the family as a function, `binomial(link = "probit")`; here the family and the link are
two words. Stata's `logit` and `probit` give the same estimates; its `probit` takes its standard
errors from the observed information, where R and this function take the expected, so the two
differ slightly; under the logit link they are one.

## Related tasks
- To fit a linear probability model of the same outcome, use `reg`.
- To read one coefficient or its interval from the fit, use `coef` and `ci`.
- To see every estimate with its statistic as a frame, use `estimates`.

## References
McCullagh, P. and Nelder, J. A. (1989), Generalized Linear Models, 2nd edition, Chapman and
Hall, chapters 2 and 4.

Agresti, A. (2013), Categorical Data Analysis, 3rd edition, Wiley, chapter 5.

## See also
reg, coef, ci, estimates, vcov

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
