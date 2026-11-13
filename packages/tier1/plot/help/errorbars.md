# errorbars

## Summary
An interval at each row's `x`, from `low:` to `high:`, with capped ends

## Description
Adds a layer of intervals to a plot: at each row an upright line reaches from `low:` to
`high:` at the row's place along `x`, with a short cap across each end. The two ends are
whatever the script computes: a mean with twice its standard error to either side, the limits
of a confidence interval a model gave, the smallest and the largest value of a group.
`m.plot(x: arm, y: mean).points().errorbars(low: lower, high: upper)` shows each mean with
its interval. It gives the plot back, so other layers and settings follow it in the chain.

An interval says how far an estimate may be trusted, which a point or a bar alone does not.
The layer draws the ends it is given and does not know what they are, so the figure must say
it: `labels(caption: "Bars: 95% confidence intervals")`. A reader who takes a standard error
for a confidence interval misjudges the evidence by half.

In R with ggplot2 this is `geom_errorbar()`, with `ymin` and `ymax` for the two ends. In
Stata it is `twoway rcap lower upper x`.

## Arguments
- p: The plot, which maps a variable to `x`. What the plot maps to `y` is not read by this
  layer.
- data: A frame for this layer alone, in place of the plot's. It holds the variable the plot
  maps to `x`, and `low:` and `high:` are read in it.
- low: The variable, or the expression, that gives the lower end of the interval at each row,
  as in `low: mean - 1.96 * se`.
- high: The same for the upper end.
- width: The width of a cap, as a share of the gap between two values of `x`, from 0 to 1.
  With 0 an interval has no caps.

## Value
The `Plot` with the layer added. The plot given is not changed.

## Details
What is drawn. Each row with a value of `x`, of `low:` and of `high:` has one interval, in the
colour of the text, as wide as the line of an axis. The `y` axis reaches to the ends of every
interval. An interval whose two ends are equal is drawn as its cap alone.

Which interval to show. The standard error measures the precision of the estimate, and one
standard error to either side covers the true value only about two times in three. A 95%
confidence interval, about two standard errors to either side for a mean of a large sample,
is the usual choice for a comparison. A standard deviation or a range describes the spread
of the data, not the precision of the mean, and belongs with a `boxplot` or the points
themselves. Two 95% intervals that overlap a little can still differ at the 5% level: the
test of a difference has its own standard error, so the figure shows each estimate and the
test is run apart.

Beside bars and boxes. Where the plot has bars side by side (`bars(position: dodge)`) or
boxes, and it fills or colours by categories, the intervals at one value of `x` stand side by
side as those marks do, each over its own bar, and a cap is as much narrower. Otherwise every
interval stands at its `x`.

Rows that are not used. A row with no value of `x`, of `low:` or of `high:`, or with an
infinite one, is left out, and a note says how many there are. An interval whose `low:` is
above its `high:` is drawn between the two all the same, and a note counts those, since the
two names are then most likely exchanged.

What the scene records. Each layer of intervals says in the scene of the plot what its
intervals are. Here it says `given`: the two ends are the script's. The printed plot shows it
with the width of the caps.

With other layers. `points()` before `errorbars` shows the estimate at the middle of its
interval. `coord(flip)` lays the intervals along the page, which gives long names their room,
as in a plot of coefficients. `hline(at: 0)` adds the line of no effect. In a plot parted
into panels by `facet` each panel has the intervals of its rows.

## References
Cumming, Fidler and Vaux (2007), Error bars in experimental biology, The Journal of Cell
Biology 177, 7–11.
Belia, Fidler, Williams and Cumming (2005), Researchers misunderstand confidence intervals
and standard error bars, Psychological Methods 10, 389–396.

## See also
plot, points, bars, ribbon, boxplot, coord

## Examples
> let m = frame(arm = ["a", "b", "c"], mean = [2.0, 3.5, 3.0], se = [0.25, 0.5, 0.25])
> m.plot(x: arm, y: mean).points().errorbars(low: mean - 1.96 * se, high: mean + 1.96 * se)
plot · m · mean against arm

  axis   title   scale      limits             breaks
  x      arm     category   0.4 to 3.6         a, b, c
  y      mean    linear     1.3615 to 4.6285   1.5, 2, 2.5, 3, 3.5, 4, 4.5

  layer   mark        data   marks   settings
      1   points      m          3   size 4.5, alpha 1
      2   errorbars   m          3   low mean - 1.96 * se, high mean + 1.96 * se, interval given, width 0.3
> let g = frame(arm = ["a", "a", "b", "b"], sex = ["f", "m", "f", "m"], mean = [2.0, 3.0, 4.0, 2.5], lower = [1.5, 2.2, 3.1, 1.9], upper = [2.5, 3.8, 4.9, 3.1])
> g.plot(x: arm, y: mean, fill: sex).bars(position: dodge).errorbars(low: lower, high: upper)
plot · g · mean against arm

  axis   title   scale      limits            breaks
  x      arm     category   0.4 to 2.6        a, b
  y      mean    linear     -0.245 to 5.145   0, 1, 2, 3, 4, 5

  aesthetic   title   scale      values   shown as
  fill        sex     category   f, m     orange, sky blue

  layer   mark        data   marks   settings
      1   bars        g          4   width 0.9, position dodge, alpha 1
      2   errorbars   g          4   low lower, high upper, interval given, width 0.3
