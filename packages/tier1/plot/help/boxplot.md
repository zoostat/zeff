# boxplot

## Summary
A box plot layer: a box over the values of `y` for each value of `x`

## Description
Draws, for each value of the variable mapped to `x`, a box over the values of `y` there: the
box holds the middle half of the values, a line across it is the median, two whiskers reach
to the furthest values that are not far from the box, and each value beyond them is a point.
`w.plot(x: industry, y: wage).boxplot()` has a box of `wage` for each industry;
`w.plot(y: wage).boxplot()` has one box of all the rows. It gives the plot back, so other
layers and settings follow it in the chain.

A box plot puts many groups side by side in little room: where their middles lie, how far
each spreads, whether it is skewed, and which values stand apart. It hides the shape within
the box, two peaks look like one, so for two or three groups a `histogram` or a `density`
of each says more.

In R this is `boxplot()`, and with ggplot2 `geom_boxplot()`. In Stata it is `graph box`.

## Arguments
- p: The plot, which maps a variable of numbers to `y`, and to `x` the variable with a box for
  each of its values, or nothing for one box.
- data: A frame for this layer alone, in place of the plot's. It holds the variables the
  plot maps.
- outliers: Whether the values beyond the whiskers are drawn, `true` or `false`. With `false`
  they are left out, the axis is that of the boxes and their whiskers, and a note says how
  many are not drawn.

## Value
The `Plot` with the layer added. The plot given is not changed.

## Details
The five numbers. The line in the box is the median. The box runs from the lower hinge to the
upper hinge: the medians of the lower and of the upper half of the values in order, the
middle value counting in both halves where their number is odd. These are Tukey's hinges, as
R's `fivenum` and `boxplot.stats` compute them; they are close to the quartiles and not
always equal to them. With `H` the height of the box, a whisker runs from each hinge to the
furthest value within `1.5 H` of it. A value further off than that is beyond the whiskers and
is drawn as a point of its own.

Values beyond the whiskers are not errors. In a sample from a normal distribution about seven
values in a thousand lie beyond them, and in a skewed one many more: of 2,152 hourly wages
78 do. They are the values to look at, not the values to remove.

What other programs do. R's `boxplot` draws these numbers, with a short line across the end
of each whisker. ggplot2 takes the quartiles by its usual rule in the place of the hinges,
which differ a little in small groups. Stata's `graph box` takes its own quartiles and the
same whiskers.

Groups and panels. Where the plot fills by categories, each value of `x` has a box for each
category that is there, side by side, the first at the left, in the colour of its category;
a box without a fill is in the colour of the page with a line round it. A box is three
quarters of the gap between two values of `x` wide, and boxes side by side share that. In a
plot parted into panels by `facet` each panel has the boxes of its rows.

Rows that are not used. A row with no value of `y`, or of `x` where one is mapped, is left
out, and a note says how many there are.

With other layers. `points(alpha: 0.3, jitter: 0.2)` after `boxplot(outliers: false)` shows
every row over its box. `coord(flip)` lays the boxes along the page, which gives long names
their room.

The printed plot names the layer a box plot, with the number of its boxes and of the values
beyond their whiskers.

## References
Tukey (1977), Exploratory Data Analysis, Addison-Wesley, Reading, Massachusetts.
McGill, Tukey and Larsen (1978), Variations of box plots, The American Statistician 32,
12–16.

## See also
plot, histogram, density, points, coord

## Examples
> let d = frame(score = [1, 2, 2, 3, 3, 3, 4, 4, 5, 9], arm = ["a", "b", "a", "b", "a", "b", "a", "b", "a", "b"])
> d.plot(x: arm, y: score).boxplot()
plot · d · score against arm

  axis   title   scale      limits       breaks
  x      arm     category   0.4 to 2.6   a, b
  y      score   linear     0.6 to 9.4   2, 4, 6, 8

  layer   mark      data   marks   settings
      1   boxplot   d          2   1 value beyond the whiskers
> d.plot(y: score).boxplot(outliers: false)
plot · d · score

  axis   title   scale      limits       breaks
  x              category   0.4 to 1.6
  y      score   linear     0.8 to 5.2   1, 2, 3, 4, 5

  layer   mark      data   marks   settings
      1   boxplot   d          1   values beyond the whiskers not drawn
note: 1 value beyond the whiskers is not drawn (line 3)
