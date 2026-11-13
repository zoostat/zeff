# bar

## Summary
A bar chart of a frame: a bar for each value of a variable, as high as its rows

## Description
Counts the rows of a frame at each value of a variable and draws a bar for each, as high as
the count. `w.bar(industry)` shows how many rows each industry has. It is the plot of a
variable of categories: where the rows lie, which categories are common and which are rare.

It is a shorthand of the grammar: `w.bar(industry, color: union)` is
`w.plot(x: industry, color: union).bars()`, the same `Plot` value. So everything a plot takes
may follow it in the chain: `coord(flip)` for bars along the page, which gives long names
their room, `labels`, `theme`, `export`.

`color:` parts each bar by the categories of a second variable, each part in its colour,
stacked on one another, with a legend. `by:` gives each value of a variable a panel of its
own, as `facet(by:)` does.

In R this is `barplot(table(x))`, and with ggplot2 `geom_bar()`. In Stata it is
`graph bar (count), over(x)`.

## Arguments
- data: The frame whose rows are counted.
- var: The variable with a bar for each of its values: categories, or a number with few
  values.
- color: A variable shown as categories; each bar is parted by them, each part in its colour.
  A number without value labels parts nothing, since a bar has one fill: the bars are drawn
  whole, and the printed plot says that no layer draws the colour.
- by: A variable, or several in brackets, whose values part the plot into panels with the
  same axes.

## Value
A `Plot`, the one its grammar form gives. It is drawn where it is printed and may be bound,
extended and exported.

## Details
What is counted. Each row with a value of `var` counts once, and the `y` axis is named
`count`. A row with `NA` in `var` is left out, and a note says how many there are. The bars
start at zero, since the eye compares their lengths.

A bar for a value that is computed. The shorthand counts rows. For bars as high as a mean, a
total or a share, compute the value first and write the grammar form, in which a variable
mapped to `y` gives the height: `w.aggregate(wage = mean(wage), by: industry)` and then
`plot(x: industry, y: wage).bars()`.

Side by side, and as shares. The parts of a bar are stacked. The grammar form arranges them
otherwise: `w.plot(x: industry, color: union).bars(position: dodge)` for bars side by side,
and `position: fill` for each bar as shares of its total.

A number with many values. A bar for each value of a measured variable is seldom wanted; a
`histogram` counts its values in bins.

A refusal of the shorthand is the refusal of `plot`, of `bars` or of `facet`, and points at
the argument of `bar` that caused it.

## See also
plot, bars, facet, histogram, tabulate, coord

## Examples
> let d = frame(arm = ["a", "b", "a", "c", "a", "b", "a", "c"], sex = ["f", "f", "m", "m", "f", "m", "m", "f"])
> d.bar(arm)
plot · d · arm

  axis   title   scale      limits        breaks
  x      arm     category   0.4 to 3.6    a, b, c
  y      count   linear     -0.2 to 4.2   0, 1, 2, 3, 4

  layer   mark   data   marks   settings
      1   bars   d          3   width 0.9, alpha 1
> d.bar(arm, color: sex)
plot · d · arm

  axis   title   scale      limits        breaks
  x      arm     category   0.4 to 3.6    a, b, c
  y      count   linear     -0.2 to 4.2   0, 1, 2, 3, 4

  aesthetic   title   scale      values   shown as
  color       sex     category   f, m     orange, sky blue
  fill        sex     category   f, m     orange, sky blue

  layer   mark   data   marks   settings
      1   bars   d          6   width 0.9, alpha 1
