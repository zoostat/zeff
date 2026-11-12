# scale_y

## Summary
The scale of the y axis: its kind, limits, breaks and labels

## Description
Says how the `y` axis of a plot shows its numbers. Without it the axis is linear, runs a
little beyond the smallest and the largest value drawn, and has its breaks at round numbers.
`p.scale_y(...)` changes any of the four: the kind of scale, where the axis starts and ends,
where its ticks stand, and what is written at them. It gives the plot back, so it is written
in the chain with the marks, before or after them.

The kind comes first and may be written by itself: `p.scale_y(log)`. On a `log` scale equal
distances are equal ratios, so values that spread over several powers of ten, incomes,
populations, prices over many years, can be read at both ends, and a constant rate of growth
is a straight line. A `sqrt` scale is between the two: it spreads the small values and holds
zero, which suits counts. `linear` is the usual scale.

`limits:` sets where the axis starts and ends, as in `limits: [0, 100]`. `breaks:` sets
where the ticks and the gridlines stand, and `labels:` what is written at each break.

`scale_x` is the same for the other axis.

In R with ggplot2 this is `scale_y_continuous()` with its `transform`, `breaks` and
`labels`, or `scale_y_log10()` and `scale_y_sqrt()` for the kinds, and
`coord_cartesian(ylim = ...)` for limits that drop no row. In Stata it is
`yscale(log range(...))` with `ylabel(...)`.

## Arguments
- p: The plot whose `y` axis is set. The axis must be one of numbers, not of categories or
  of dates.
- type: The kind of scale: `linear`, `log` or `sqrt`. `date` is for an axis of dates.
- limits: A list of two numbers, where the axis starts and where it ends. `NA` in the place
  of one leaves that end to the values drawn, as in `limits: [0, NA]`.
- breaks: A list of numbers: the values at which the axis has a tick, a gridline and a label.
- labels: A list of text, one for each of `breaks:` in the order they are written: what is
  written at each break in the place of the number.

## Value
The `Plot` with the scale of its `y` axis set. The plot given is not changed. What a
later `scale_y` says takes the place of all that an earlier one said.

## Details
Limits show; they do not select. `limits:` sets the visible range and nothing more. No row is
dropped for it: a mark beyond a limit is cut off at the edge of the panel, a line runs on to
the edge towards a point that is not seen, and a note counts the marks beyond the limits. A
layer that summarises its rows, a count or a mean, uses every row, whatever the limits. To
leave rows out of the figure altogether, take them out of the frame first, with `where`. A
limit that is given is that end of the axis exactly, with no margin beyond it.

The log scale. A value at or below zero has no logarithm and so no place on a log scale: such
a plot is refused, with the variable and the first of its rows named, whether the scale or
the layer comes last in the chain. Bars and areas are refused on a log `y` axis as well, since
they reach down to zero; `points` and `lines` show such values. The margin beyond the
smallest and the largest value is taken in the logarithm, so it is the same share of the
panel at both ends. The breaks are powers of ten where the axis spans three or more of them,
and 1, 2 and 5 times a power of ten where it spans fewer; under one power of ten they are
round numbers, as on a linear axis.

The square-root scale. A value below zero has no place on it; zero has. The breaks are round
numbers, which stand closer together as they grow.

Breaks and labels. Breaks are put in increasing order, and a break outside the limits is not
shown. `labels:` needs `breaks:`, and one label for each. Without `labels:` a break is written
as a number, as tables print it. A variable with value labels, on a linear axis that the
script gives no breaks, has its ticks at the labelled values, each written as its label, so a
variable coded 0 and 1 reads `no` and `yes`.

Categories. An axis of categories, of a Str or a Category variable, has a place for each
category and no other scale: `scale_y` is refused on it.

Dates. An axis of dates, of a Date or a DateTime variable, is placed in proportion to the
time and chooses its limits and its breaks from the dates drawn. A log or a square-root scale
is refused on it, and so, in this build, are `limits:`, `breaks:` and `labels:`.

## See also
scale_x, plot, points, where

## Examples
> let d = frame(year = [1, 2, 3, 4], sales = [3, 30, 250, 4000])
> d.plot(x: year, y: sales).points().lines().scale_y(log)
plot · d · sales against year

  axis   title   scale    limits               breaks
  x      year    linear   0.85 to 4.15         1, 1.5, 2, 2.5, 3, 3.5, 4
  y      sales   log      2.09351 to 5732.01   10, 100, 1000

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1
      2   lines    d          4   width 1, 1 line
