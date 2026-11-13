# plot

## Summary
The default plot of a fitted model: its coefficients, or its event study

## Description
Draws the figure a fit is most often shown by. For a regression it is the coefficient plot:
`m.plot()` is `m.coef_plot()`, the estimates as points with their confidence intervals, a
term on each line. For a regression with the terms of an event study, `event(year, treated)`
in its formula, it is the event-study plot: the estimate of each period relative to the
event, in the order of the periods along the foot, each with its confidence interval, and a
line at zero.

It gives a `Plot`, so everything a plot takes may follow it in the chain: `labels`, `theme`,
`export`.

`plot` is the default view of a result, one for each type of result. A plot derived from a
result in another way is a method named for the kind of figure, as `coef_plot` is.

In R this is `plot(m)` for the method a model's class defines, and `fixest::iplot(m)` for an
event study. In Stata it is `coefplot`, and `event_plot` for an event study.

## Arguments
- fit: A fitted model, as `reg` gives it.

## Value
A `Plot` of three layers: the line at zero, the intervals and the points. The caption says
what the bars are, "Bars: 95% confidence intervals" at the usual level; a caption given with
`labels(caption:)` takes its place.

## Details
The event study. The terms `event=-3`, `event=0`, `event=2` of the fit are the periods before
and after the event; the plot places each estimate at its period, so the path of the effect
is read from left to right. The base period, the one left out of the model (the period before
the event unless `ref:` names another), has no estimate: its place on the axis is empty, and
the other estimates are differences from it. The other terms of the model, the controls, are
not drawn; `coef_plot` draws every term.

Reading it. Estimates before the event that lie near zero, with intervals across it, are what
parallel trends would give; a drift before the event is a warning that the groups were moving
apart already. The intervals are those of the fit, at its level and with its standard errors,
clustered where the fit is. Each interval is of one period alone: whether the periods before
the event are zero together is a joint test, which `test` runs.

The coefficient plot is described in the help of `coef_plot`: the intercept is left out, and
the terms stand in the order of the table.

The scene records the intervals as `ci 0.95`, with the level of the fit. A fit of the
intercept alone has nothing to draw and is refused.

## See also
coef_plot, reg, event, estimates, test

## Examples
> let w = dataset(wages)
> let m = w.reg(log(wage) ~ educ + exper + union)
> m.plot()
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
