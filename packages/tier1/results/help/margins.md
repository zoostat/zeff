# margins

## Summary
Marginal effects and adjusted predictions, with delta-method standard errors

## In brief
- `m.margins(educ)`: the average effect of `educ` on the outcome, through every term it enters.
- `m.margins([educ, exper, female])`: several variables in one table.
- A number gives a slope; a Bool, a 0/1 variable or a category a difference between its values.
- `at: {exper: 10}` holds a variable at a value, `at: {exper: [5, 10, 20]}` at each in turn; `type: mem` takes the effects at the means.
- `m.margins()`, with no variable, gives the adjusted prediction, at the values of `at:` if given.
- After `glm` the effects are changes in the probability; the standard errors follow the fit's own.

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

`at:` asks the question at chosen values: with `at: {exper: 10}` every row is given ten years of
experience, its other variables left as they are, and the effects are averaged over those rows.
`type: mem` asks it at one point, the means of the variables, in the place of the average over
the rows. With no variable named, the answer is not an effect but the adjusted prediction: the
average of the model's predictions for the rows, or the prediction at the means, with the
variables of `at:` held at its values.

`margins` reads a fit of `reg` or of `glm`. After `glm` the coefficients are on the scale of the
link, log-odds under the logit, and say little by themselves; `margins` gives every effect and
every prediction on the scale of the probability.

## Arguments
- fit: The fit, from `reg` or `glm`.
- vars: The variable whose effect is wanted, or a list of several, named without quotes. A variable is named, not a term: `exper`, not `exper^2`. Left out, the result is the adjusted prediction.
- at: The values to hold variables at, as a record: `{exper: 10, industry: "Mining"}`. A number is held at a number and a category at one of its levels; every other variable stays as it is. A list of values, `{exper: [5, 10, 20]}`, gives the result at each.
- type: `ame`, the average over the rows the fit used, or `mem`, at the means of the variables; `ame` unless given.

## Value
A `TestResult` whose table has a row for each effect: the variable, what is compared (`dy/dx`
for a slope, the two values or the two levels for a difference), the estimate, its standard
error, the t value on the fit's degrees of freedom (the z value, after `glm`), the p-value and
the interval at the fit's level. The lines above the table say what the effects are averaged over, or that they are at
the means, and what `at:` holds; lines under it say which terms a slope was taken through. With
no variable named the table has the one row `prediction`. `std_errors` and `ci` read one row by
its name as `p_values` does, and `report` sets the result beside its fit, or beside other
results of `margins`, in the layout of a journal table. `estimates` gives the
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

### Held values
A variable named in `at:` is set to the value given in every row before anything is computed;
every other variable keeps what the row has. A number is held at a number, and a category at
one of the levels it has in the model, written as text. The slope in a variable may be asked
where the variable itself is held, `m.margins(exper, at: {exper: 10})`, the slope at ten years.
A 0/1 variable or a category cannot be both asked for and held, since its effect is the
difference between its values (Z0401).

A variable given a list of values is held at each in turn, and the table has a row for each,
with the value in a column of its own. Several variables with lists give every combination of
their values, the first variable changing slowest. Such a row is named with its values,
`exper @ exper=5` or `prediction @ exper=5, industry=Mining`, for `p_values` and `estimates`.
The rows are separate statements, each with its own interval; they are not tested against one
another.

### At the means
`type: mem` takes every effect at one point in the place of an average over the rows: each
number at its mean over the rows the fit used, and each category at the shares of its levels,
so that the indicator of a level stands at its mean too. A Bool or a 0/1 variable is a number
here, at its mean, the share of ones. The point need not look like any row: 46 per cent
female. A variable of `at:` is held at its value and not at its mean. Under weights the means
and the shares are weighted.

Where every effect is linear in the other variables, as with a square or an interaction of two
variables, the effect at the means is the average effect. They differ where a term bends in
another way: the slope through `log(hours)` at the means is the coefficient over the mean of
`hours`, and its average is the coefficient times the mean of 1 over `hours`.

### Adjusted predictions
With no variable named, the result is the prediction of the model: averaged over the rows
(`type: ame`), or at the means (`type: mem`), with the variables of `at:` held. With nothing
held, the average prediction is the mean of the fitted values. After `fe:` a prediction would
need the level of each fixed effect and is refused (Z0401); the effects of variables are given
as before.

### After glm
With p the probability the model gives a row, a function of the row's linear combination of the
regressors, the slope in a number is the slope of p: the slope of that combination, through
every term the variable enters, times how fast the probability rises with it at the row, which
is p(1 − p) under the logit and the normal density under the probit. The effect of a 0/1
variable or of a level is the difference between the probabilities at the two values, and the
adjusted prediction is the probability itself. Each differs from row to row even in a model
without interactions, so here the average over the rows and the value at the means are not the
same thing: `type: mem` gives the effect for the one point of the means, the average of
nothing. With nothing held, the average prediction of a logit is the share of ones.

### Standard errors
In a linear model every effect is a weighted sum of the coefficients, the weights being the
average over the rows of the derivative (or of the difference) of each column of the model
matrix. Its variance is that sum's variance under the fit's own variance matrix, so a fit with
robust or clustered errors gives effects with robust or clustered errors. The regressors are
held as they are in the sample: the standard error does not allow for another sample having
other values of them. The t value and the interval use the fit's degrees of freedom. After
`glm` an effect is not a weighted sum of the coefficients but a smooth function of them, and its
variance is taken through the function's gradient at the estimates; the z value and the
interval are the normal distribution's, as the fit's own table has them.

### Weights and fixed effects
Under weights the average over the rows is weighted by them, each row counted as its weight
says. After `fe:` the effects are the same as with the fixed effects written as a category,
since a slope and a difference do not depend on the level of an effect.

### When there is no effect
A variable that enters the model both as a number and as a category has no one effect and is
refused (Z0417), as is a slope that is not finite in some row, such as that of `sqrt(x)` where
`x` is 0, and a term with no value at the values asked for, such as `log(x)` with `x` held at 0. An effect that rests on a term the fit omitted as collinear is shown as `NA`, with a
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
| m.margins(x, at: {z: 10}) | marginaleffects::avg_slopes(m, variables = "x", newdata = transform(d, z = 10)) | margins, dydx(x) at(z = 10) |
| m.margins(x, type: mem) | marginaleffects::slopes(m, variables = "x", newdata = "mean") | margins, dydx(x) atmeans |
| m.margins() | marginaleffects::avg_predictions(m) | margins |
| m.margins(at: {z: 10}) | marginaleffects::avg_predictions(m, newdata = transform(d, z = 10)) | margins, at(z = 10) |
| m.margins(at: {z: [5, 10, 20]}) | marginaleffects::avg_predictions(m, variables = list(z = c(5, 10, 20))) | margins, at(z = (5 10 20)) |

After a logit or a probit the same calls give effects on the probability in all three: R's
`glm` with marginaleffects, Stata's `logit` or `probit` with `margins`.

Stata takes the slope through a square only when the model is written with its factor
notation, `c.exper##c.exper`; here, as in R with `I(exper^2)`, the formula's own `exper^2` is
enough. Stata treats a 0/1 variable as a number unless it is written `i.d`, and then gives the
difference; here, as in marginaleffects, a 0/1 variable always gives the difference. The
estimates agree with both. The standard errors agree with Stata's, and with marginaleffects'
when it is given the same variance matrix (`vcov =`); the p-values and intervals are from
Student's t, as Stata's are after `regress`, where marginaleffects uses the normal distribution
unless `df =` is set. After `glm` all three use the normal distribution; clustered errors carry
the factor the page of `glm` describes. Under weights Stata weights the average as here; marginaleffects does so
when `wts = TRUE`. At the means, a category stands at the shares of its levels, as under
Stata's `atmeans`; marginaleffects' `newdata = "mean"` puts a category at its commonest level,
so the two agree only for a model without categories. In R the data must be the rows the fit
used.

## Related tasks
- To read a coefficient itself, use `coef`.
- To get a ratio or another function of the coefficients with its standard error, use `derive`.
- To test a restriction on the coefficients, use `test`.
- To get each row's own prediction, use `fitted`; for new rows, `predict`.
- To see every effect as a frame at full precision, use `estimates`.
- To set the effects beside the coefficients in one table, use `report`.
- To write the table to Word or LaTeX, use `export`.

## References
Williams, R. (2012), Using the margins command to estimate and interpret adjusted predictions
and marginal effects, The Stata Journal, 12, 308–331.

Arel-Bundock, V., Greifer, N. and Heiss, A. (2024), How to interpret statistical models using
marginaleffects for R and Python, Journal of Statistical Software, 111(9).

## See also
reg, glm, coef, derive, test, predict, estimates, std_errors, ci, report, export

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

// The slope in exper at ten years of experience, the other variables as they are.
> m.margins(exper, at: {exper: 10})
margins · m · average marginal effects
reg · log(wage) ~ educ + exper + exper^2 + female
averaged over the 18,179 observations the fit used
at: exper = 10
standard errors: robust (HC1), by the delta method

  variable   contrast   estimate   std.error   t value   p value   95% CI
  exper      dy/dx        0.0289      0.0007     43.45    <0.001   [0.0276, 0.0302]

the slope of `exper` is taken through `exper` and `exper^2`

// The same effects at the means of the variables.
> m.margins([educ, exper, female], type: mem)
margins · m · marginal effects at the means
reg · log(wage) ~ educ + exper + exper^2 + female
at the means over the 18,179 observations the fit used
standard errors: robust (HC1), by the delta method

  variable   contrast        estimate   std.error   t value   p value   95% CI
  educ       dy/dx             0.0758      0.0014     56.07    <0.001   [0.0732, 0.0785]
  exper      dy/dx             0.0222      0.0004     49.61    <0.001   [0.0213, 0.0230]
  female     female − male    -0.1319      0.0064    -20.63    <0.001   [-0.1445, -0.1194]

the slope of `exper` is taken through `exper` and `exper^2`

// With no variable named, the adjusted prediction: here for a woman with 16 years of schooling.
> m.margins(at: {educ: 16, female: 1})
margins · m · average adjusted prediction
reg · log(wage) ~ educ + exper + exper^2 + female
averaged over the 18,179 observations the fit used
at: educ = 16, female = 1
standard errors: robust (HC1), by the delta method

  quantity     estimate   std.error   t value   p value   95% CI
  prediction     3.3149      0.0057    583.37    <0.001   [3.3037, 3.3260]

// After a logit: a year of experience raises the probability of union membership by 0.6 points.
> let g = wages.glm(union ~ educ + exper + exper^2 + female, family: binomial)
> g.margins([exper, female])
margins · g · average marginal effects
glm · union ~ educ + exper + exper^2 + female · binomial, logit
on the scale of the probability, averaged over the 18,390 observations the fit used
standard errors: classical, by the delta method

  variable   contrast        estimate   std.error   z value   p value   95% CI
  exper      dy/dx             0.0060      0.0005     12.33    <0.001   [0.0050, 0.0069]
  female     female − male     0.0117      0.0068      1.70     0.089   [-0.0018, 0.0251]

the slope of `exper` is taken through `exper` and `exper^2`

// The probability of union membership along experience: the prediction at each of four values.
> g.margins(at: {exper: [0, 10, 20, 30]})
margins · g · average adjusted prediction
glm · union ~ educ + exper + exper^2 + female · binomial, logit
on the scale of the probability, averaged over the 18,390 observations the fit used
at: exper = 0, 10, 20, 30
standard errors: classical, by the delta method

  exper   quantity     estimate   std.error   z value   p value   95% CI
      0   prediction     0.2618      0.0115     22.85    <0.001   [0.2394, 0.2843]
     10   prediction     0.2824      0.0043     65.56    <0.001   [0.2739, 0.2908]
     20   prediction     0.3399      0.0047     72.67    <0.001   [0.3307, 0.3491]
     30   prediction     0.4429      0.0130     34.04    <0.001   [0.4174, 0.4684]
