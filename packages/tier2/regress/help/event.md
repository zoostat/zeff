# event

## Summary
Event-study leads and lags inside a formula

## Description
A term of a formula that stands for the indicators of an event study: `event(year, treated_in)`
expands to one column per relative period `year − treated_in` present among the treated rows,
except the reference period `ref:` (the period before treatment, −1, unless given), each 1 in
the rows at that period and 0 elsewhere. `treat` holds the period in which a row's unit was
first treated, in the time variable's own type, and `NA` for a unit never treated; such rows are
0 in every column, so the never-treated units are the controls, as fixest's
`i(rel, ref = c(-1, -1000))` arranges. The columns are named like a category's levels,
`event=-3`, `event=0`, `event=2`, in period order, so that `m.coef("event=0")`, `report` and
`test` read them; combined with `fe: unit + year` and `cluster: unit` the term gives the
two-way fixed-effects event study. `event` has no value of its own, so outside a formula it is
an error that says where it belongs.

## Arguments
- time: The time variable: an Int such as a year, or a Date, whose relative periods are days.
- treat: The period each row's unit was first treated in, of the time variable's type; `NA`
  for a unit never treated.
- ref: The relative period left out as the base; −1, the period before treatment, unless given.
  A period absent from the data is an error listing the periods present (Z0507).

## Value
A term of a formula, read by the model matrix; no value outside one.

## Details
A row whose time is `NA` is `NA` in every column and is dropped with the model variables'; a
unit treated in a period outside the data's time span contributes its relative periods like any
other. When every unit is treated, or all at once, the indicators of some periods are collinear
with the time effects, and the fit drops them as collinear with a note naming each, which is the
identification the design lacks. Against fixest's `i()` with the never-treated rows given a
reference of their own, the estimates agree within the closed-form class.

## References
Bergé, L. (2018). Efficient estimation of maximum likelihood models with multiple fixed-effects:
the R package FENmlm. CREA Discussion Papers, 13. Sun, L. and Abraham, S. (2021). Estimating
dynamic treatment effects in event studies with heterogeneous treatment effects. *Journal of
Econometrics*, 225, 175–199.

## See also
reg, interact, category, report

## Examples
> let d = frame(unit = [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4], year = [2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013], treat = [2011, 2011, 2011, 2011, 2012, 2012, 2012, 2012, 2013, 2013, 2013, 2013, NA, NA, NA, NA], y = [1.0, 2.1, 2.4, 2.6, 1.5, 1.4, 2.6, 2.9, 0.8, 0.9, 1.1, 2.0, 1.2, 1.3, 1.1, 1.4])
> let m = d.reg(y ~ event(year, treat), fe: unit + year)
> m
reg · y ~ event(year, treat) · fe: unit + year
N = 16 (no singletons)
fixed effects: unit (4), year (4)
standard errors: classical

  term       estimate   std.error   t value   p value   95% CI
  event=-3    -0.1316      0.2121     -0.62     0.568   [-0.7204, 0.4571]
  event=-2    -0.0063      0.1411     -0.04     0.966   [-0.3981, 0.3854]
  event=0      1.0273      0.1242      8.27     0.001   [0.6823, 1.3722]
  event=1      1.3119      0.1738      7.55     0.002   [0.8293, 1.7944]
  event=2      1.4502      0.2510      5.78     0.004   [0.7534, 2.1470]

R² within 0.953 · R² overall 0.990
> m.coef("event=0")
1.02727
