# margins

## Summary
Average marginal effects of a fitted model, with delta-method standard errors

## In brief
- `m.margins(educ)`: the average effect of `educ` on the outcome, through every term it enters.
- `m.margins([educ, exper, female])`: several variables in one table.
- A number gives a slope; a Bool or a 0/1 variable the difference between its two values.
- A category gives a row for each level, compared with the first.
- The standard errors follow the fit's own: classical, robust or clustered.

## Description
Answers the question a table of coefficients answers only for the simplest model: by how much
does the outcome change when one variable changes, the others staying as they are? Where a
variable enters one term, as it stands, the answer is its coefficient. Where it enters several,
a square beside the variable or an interaction with another, no single coefficient is the
answer, and `margins` puts them together: after `log(wage) ~ exper + exper^2` the slope in
`exper` is the first coefficient plus twice the second times `exper`, which differs from row to
row, and `margins` reports its average over the rows the fit used.

For a number the effect is a slope, the derivative of the prediction. For a Bool or a variable
whose values are 0 and 1 it is a difference: the prediction with the variable at 1 less the
prediction with it at 0, for each row, averaged. For a category it is the same difference
between each level and the first, one row to a level.

At this release `margins` reads a fit of `reg`.

## Arguments
- fit: The fit, from `reg`.
- vars: The variable whose effect is wanted, or a list of several, named without quotes. A variable is named, not a term: `exper`, not `exper^2`.

## Value
A `TestResult` whose table has a row for each effect: the variable, what is compared (`dy/dx`
for a slope, the two values or the two levels for a difference), the estimate, its standard
error, the t value on the fit's degrees of freedom, the p-value and the interval at the fit's
level. Lines under the table say which terms a slope was taken through. `estimates` gives the
rows as a frame at full precision and `ci` the intervals; a row is named as the fit names the
term, `exper` for a number and `industry=Mining` for a level, so that `p_values` reads one
row's p-value by that name.

## Missing values
The effects are averaged over the rows the fit used, so a row the fit left out for `NA` is left
out here too.

## Details
### Slopes
The slope in a variable is taken through every term that is a function of it: the variable
itself, a power, a logarithm or another function of it, and each interaction it enters. The
derivative of each term is exact where the term is arithmetic (`+`, `-`, `*`, `/`, `^`) or one
of `log`, `exp`, `sqrt`, `log1p`, `expm1`, `log10`, `log2` and `abs`. Through any other
function it is a central difference, and a line under the table says so; that is right
wherever the term is smooth, as a hinge `max(hours - 40, 0)` is everywhere but at 40.

A power computed beforehand into a variable of its own, `add(exper2 = exper^2)`, is a regressor
like any other: the model no longer says that it is a function of `exper`, and the slope in
`exper` is then the coefficient of `exper` alone. Write the power in the formula.

### Differences
A Bool, or a number whose values in the rows used are all 0 or 1, is compared at its two
values, and the table names them by their value labels where the variable carries them. The
levels of a category are each compared with the first, whatever contrast the category is
coded by. Each difference is computed for every row, the other variables as that row has them,
and averaged: where the variable is interacted with another, the difference varies from row to
row, and the average is over the rows as they are.

### Standard errors
In a linear model every effect is a weighted sum of the coefficients, the weights being the
average over the rows of the derivative (or of the difference) of each column of the model
matrix. Its variance is that sum's variance under the fit's own variance matrix, so a fit with
robust or clustered errors gives effects with robust or clustered errors. The regressors are
held as they are in the sample: the standard error does not allow for another sample having
other values of them. The t value and the interval use the fit's degrees of freedom.

### Weights and fixed effects
Under weights the average over the rows is weighted by them, each row counted as its weight
says. After `fe:` the effects are the same as with the fixed effects written as a category,
since a slope and a difference do not depend on the level of an effect.

### When there is no effect
A variable that enters the model both as a number and as a category has no one effect and is
refused (Z0417), as is a slope that is not finite in some row, such as that of `sqrt(x)` where
`x` is 0. An effect that rests on a term the fit omitted as collinear is shown as `NA`, with a
line saying so.

## A common mistake
Asking for the effect of a variable that is not in the model:

> let m = dataset(wages).reg(log(wage) ~ educ + exper + exper^2 + female)
> m.margins(hours)
error[Z0414]: `hours` is not a variable of this model
  = help: the variables of the model are `educ`, `exper`, `female`

The message lists the variables the model's terms are functions of. A term is not asked for by
its own name: the slope through `exper^2` is part of `margins(exper)`.

## Coming from R or Stata
| m.margins(x) | marginaleffects::avg_slopes(m, variables = "x") | margins, dydx(x) |
| m.margins(d), d a 0/1 variable | marginaleffects::avg_comparisons(m, variables = "d") | margins, dydx(d), with i.d in the model |
| m.margins(g), g a category | marginaleffects::avg_comparisons(m, variables = list(g = "reference")) | margins, dydx(g), with i.g in the model |

Stata takes the slope through a square only when the model is written with its factor
notation, `c.exper##c.exper`; here, as in R with `I(exper^2)`, the formula's own `exper^2` is
enough. Stata treats a 0/1 variable as a number unless it is written `i.d`, and then gives the
difference; here, as in marginaleffects, a 0/1 variable always gives the difference. The
estimates agree with both. The standard errors agree with Stata's, and with marginaleffects'
when it is given the same variance matrix (`vcov =`); the p-values and intervals are from
Student's t, as Stata's are after `regress`, where marginaleffects uses the normal distribution
unless `df =` is set. Under weights Stata weights the average as here; marginaleffects does so
when `wts = TRUE`.

## Related tasks
- To read a coefficient itself, use `coef`.
- To get a ratio or another function of the coefficients with its standard error, use `derive`.
- To test a restriction on the coefficients, use `test`.
- To get each row's prediction, use `fitted`; for new rows, `predict`.
- To see every effect as a frame at full precision, use `estimates`.

## References
Williams, R. (2012), Using the margins command to estimate and interpret adjusted predictions
and marginal effects, The Stata Journal, 12, 308–331.

Arel-Bundock, V., Greifer, N. and Heiss, A. (2024), How to interpret statistical models using
marginaleffects for R and Python, Journal of Statistical Software, 111(9).

## See also
reg, coef, derive, test, predict, estimates, ci

## Examples
// The average effects after a model with a square: the slope in exper is not its coefficient.
> let wages = dataset(wages)
> let m = wages.reg(log(wage) ~ educ + exper + exper^2 + female, se: robust)
> m.margins([educ, exper, female])
margins · m · average marginal effects
reg · log(wage) ~ educ + exper + exper^2 + female
averaged over the 18,179 observations the fit used
standard errors: robust (HC1), by the delta method

  variable   contrast        estimate   std.error   t value   p value   95% CI
  educ       dy/dx             0.0758      0.0014     56.07    <0.001   [0.0732, 0.0785]
  exper      dy/dx             0.0222      0.0004     49.61    <0.001   [0.0213, 0.0230]
  female     female − male    -0.1319      0.0064    -20.63    <0.001   [-0.1445, -0.1194]

the slope of `exper` is taken through `exper` and `exper^2`

// The coefficient of exper alone is the slope where exper is 0, almost twice the average.
> m.coef(exper)
0.0420585

// A category: each industry against the first, Agriculture.
> let c = wages.reg(log(wage) ~ educ * female + industry)
> c.margins(industry)
margins · c · average marginal effects
reg · log(wage) ~ educ * female + industry
averaged over the 18,201 observations the fit used
standard errors: classical, by the delta method

  variable   contrast                              estimate   std.error   t value   p value   95% CI
  industry   Mining − Agriculture                    0.2111      0.0379      5.56    <0.001   [0.1367, 0.2854]
  industry   Utilities − Agriculture                 0.2345      0.0310      7.56    <0.001   [0.1737, 0.2953]
  industry   Construction − Agriculture              0.2404      0.0232     10.38    <0.001   [0.1950, 0.2858]
  industry   Manufacturing − Agriculture             0.2161      0.0218      9.91    <0.001   [0.1733, 0.2588]
  industry   Retail − Agriculture                    0.1424      0.0216      6.59    <0.001   [0.1000, 0.1847]
  industry   Transport − Agriculture                 0.1736      0.0247      7.03    <0.001   [0.1252, 0.2220]
  industry   Finance − Agriculture                   0.2827      0.0230     12.30    <0.001   [0.2377, 0.3277]
  industry   Services − Agriculture                  0.0963      0.0218      4.43    <0.001   [0.0537, 0.1389]
  industry   Education − Agriculture                 0.2119      0.0228      9.30    <0.001   [0.1672, 0.2565]
  industry   Health − Agriculture                    0.2032      0.0222      9.14    <0.001   [0.1596, 0.2467]
  industry   Public administration − Agriculture     0.2697      0.0233     11.59    <0.001   [0.2241, 0.3153]

// The effect of educ through its interaction with female, as a frame at full precision.
> c.margins(educ).estimates()
frame · 1 row × 9 variables

      term    estimate    std_error   statistic      df   df2   p_value    ci_lower   ci_upper
  1   educ   0.0545994   0.00136249     40.0731   18186    NA         0   0.0519288    0.05727
