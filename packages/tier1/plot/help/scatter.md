# scatter

## Summary
A scatter plot of a frame: a point for each row, with a fitted line if asked

## Description
Draws `y` against `x`, a point for each row, in one call: `w.scatter(educ, wage)`. It is the
plot to begin with for two measured variables: whether one rises with the other, how closely,
along a line or a curve, and which rows stand apart.

`fit: linear` adds the least-squares line of `y` on `x` over the points, with the 95%
confidence band of the line under them. `color:` colours the points by a variable, and with
categories each has a line and a band of its own. `by:` gives each value of a variable a
panel of its own, as `facet(by:)` does, each with its own line.

Without a fit it is a shorthand of the grammar: `w.scatter(educ, wage, color: union)` is
`w.plot(x: educ, y: wage, color: union).points()`, the same `Plot` value. So everything a plot
takes may follow it in the chain: `scale_y(log)`, `labels`, `theme`, `export`.

In R this is `plot(x, y)`, with `abline(lm(y ~ x))` for the line; with ggplot2 it is
`geom_point()` and `geom_smooth(method = "lm")`. In Stata it is `scatter y x`, and
`twoway (lfitci y x) (scatter y x)` with the line and its band.

## Arguments
- data: The frame whose rows are drawn.
- x: The variable, or the expression, along the foot of the plot, as in `educ` or
  `log(hours)`.
- y: The variable, or the expression, up the side.
- fit: `none` for the points alone, or `linear` for the least-squares line of `y` on `x` and
  its band as well.
- color: A variable that colours the points: each category in its colour, or a number on a
  ramp of colours. A fitted line is drawn for each category; with a number on a ramp there is
  one line, of all the rows.
- by: A variable, or several in brackets, whose values part the plot into panels with the
  same axes.

## Value
A `Plot`. Without a fit it is the one its grammar form gives. It is drawn where it is printed
and may be bound, extended and exported.

## Details
The line. With `fit: linear` the line is the least-squares line of `y` on `x`, the one
`reg(y ~ x)` estimates, fitted by the same QR decomposition. It is drawn from the smallest
value of `x` to the largest and no further, since beyond the data it is a guess.

The band. At each place along `x` the band is the 95% confidence interval of the fitted
value: the fitted value, and to either side the 0.975 quantile of Student's t on `n - 2`
degrees of freedom times the standard error of the fitted value there. It is narrowest at the
mean of `x` and widens towards the ends. It is the classical band: it takes the errors to
have one variance and to be independent, as the ordinary standard errors of `reg` do. It says
where the line may lie, not where a new row may fall, which is a much wider interval. For a
band from robust or clustered standard errors, fit the model with `reg` and draw its
predictions.

The figure says what the band is: where the script gives no caption, the caption reads
"Band: 95% confidence interval of the fitted line", and the scene of the plot records the
interval as `ci 0.95`. A caption given with `labels(caption:)` takes its place.

Groups and panels. With `color:` by categories, and in each panel of `by:`, the line is that
of the rows of the group alone, as if the model were fitted to each group apart. A group with
fewer than three rows, or with one value of `x`, has no line, and a note counts such groups.

Other scales. The line is fitted to the values as the axes show them. After `scale_y(log)`
it is the line of the logarithm of `y` on `x`, drawn straight on the log axis, with its band;
the same holds for `scale_x(log)` and for `sqrt`. Along `x` a date is counted in days. With
categories along either axis there is no line to fit, and `fit: linear` is refused.

A straight line is not always the pattern. The four data sets of Anscombe have the same line
and the same correlation and look nothing alike; the points are there to be looked at before
the line is believed.

Rows not drawn. A row with `NA` or an infinite value of `x` or `y` is left out of the points
and of the fit, and a note says how many there are.

A refusal of the shorthand is the refusal of `plot`, of `points` or of `facet`, and points
at the argument of `scatter` that caused it.

## References
Anscombe (1973), Graphs in statistical analysis, The American Statistician 27, 17–21.
Weisberg (2014), Applied Linear Regression, fourth edition, Wiley, Hoboken.

## See also
plot, points, reg, line, facet, correlation

## Examples
> let d = frame(dose = [1, 2, 3, 4, 5, 6, 7, 8], response = [2.1, 3.9, 6.2, 7.8, 9.0, 11.5, 13.1, 16.4])
> d.scatter(dose, response)
plot · d · response against dose

  axis   title      scale    limits            breaks
  x      dose       linear   0.65 to 8.35      2, 4, 6, 8
  y      response   linear   1.385 to 17.115   2, 4, 6, 8, 10, 12, 14, 16

  layer   mark     data   marks   settings
      1   points   d          8   size 4.5, alpha 1
> d.scatter(dose, response, fit: linear)
plot · d · response against dose

  axis   title      scale    limits                breaks
  x      dose       linear   0.65 to 8.35          2, 4, 6, 8
  y      response   linear   0.349186 to 17.1643   5, 10, 15

  layer   mark       data   marks   settings
      1   fit band   d         80   95% confidence band, alpha 0.2, 1 band
      2   points     d          8   size 4.5, alpha 1
      3   fit line   d         80   least squares of response on dose, width 1, 1 line

caption: Band: 95% confidence interval of the fitted line
