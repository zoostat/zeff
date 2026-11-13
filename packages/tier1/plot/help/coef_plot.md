# coef_plot

## Summary
The coefficients of a fitted model as points with their confidence intervals

## Description
Draws the estimates of a fitted model, a term on each line: a point at the estimate and a
bar from the lower limit of its confidence interval to the upper, with a line at zero.
`m.coef_plot()` after `w.reg(log(wage) ~ educ + exper + union)` shows at a glance which
coefficients are far from zero, which are estimated closely and which are not, and which
intervals reach across zero. It is the figure of the table `m` prints.

The terms stand in the order of the table, the first at the top. The intercept is left out:
it is on another scale than the slopes and is seldom what the figure is for. The intervals
are those of the fit, at its `level:` and with its standard errors, robust or clustered
where the fit is.

It gives a `Plot`, so everything a plot takes may follow it in the chain: `labels`, `theme`,
`export`.

A plot derived from a result is a method named for the kind of figure, `coef_plot` here, one
for each kind; `fit.plot()` is the default view of a result.

In R this is `modelsummary::modelplot(m)` or `coefplot::coefplot(m)`. In Stata it is
`coefplot`.

## Arguments
- fit: A fitted model, as `reg` gives it.

## Value
A `Plot` of three layers: the line at zero, the intervals and the points. The estimates are
along the foot and the terms down the side. The caption says what the bars are, "Bars: 95%
confidence intervals" at the usual level; a caption given with `labels(caption:)` takes its
place.

## Details
What is drawn. The frame of the plot has a row for each term but the intercept, with the
variables `term`, `estimate`, `ci_lower` and `ci_upper`, the numbers `estimates()` gives. The
plot is `plot(x: term, y: estimate)` with `hline(at: 0)`, `errorbars(low: ci_lower, high:
ci_upper, width: 0)` and `points()`, turned by `coord(flip)` so that the names of the terms
have their room. The scene records the intervals as `ci 0.95`, with the level of the fit.

Reading it. An interval that does not reach zero belongs to a coefficient that differs from
zero at the level of the fit; it says nothing of whether the coefficient is large. Two
intervals that overlap may still belong to coefficients that differ: the test of their
difference has its own standard error, and `test` runs it. Coefficients are in the units of
their variables, so the lengths of the bars of two terms are compared only where the
variables share a unit; the columns of one category do, and a category with many levels is
where the figure serves best.

Many terms. The terms of a category, as `industry=Mining`, stand one to a line under their
names. A term the fit omitted as collinear has no estimate and no line.

With fixed effects there is no intercept, and the terms are those estimated; the effects
themselves are not coefficients of the table and are not drawn.

## See also
reg, estimates, ci, errorbars, plot, test

## Examples
> let w = dataset(wages)
> let m = w.reg(log(wage) ~ educ + exper + union)
> m.coef_plot()
plot · m · estimate against term

  axis   title      scale      limits                    breaks
  x                 category   0.4 to 3.6                union, exper, educ
  y      estimate   linear     -0.00506249 to 0.106312   0, 0.02, 0.04, 0.06, 0.08, 0.1

  layer   mark        data   marks   settings
      1   hline       m          1   at 0, width 0.5, style dashed
      2   errorbars   m          3   low ci_lower, high ci_upper, interval ci 0.95, width 0
      3   points      m          3   size 4.5, alpha 1

caption: Bars: 95% confidence intervals
coordinates: flipped, `x` up the side and `y` along the foot
