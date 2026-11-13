# boxplot

## Summary
A box plot of a variable of a frame, with a box for each group

## Description
Draws a box over the values of a variable, in one call: `w.boxplot(wage)` has one box of all
the rows, and `w.boxplot(wage, group: industry)` a box for each industry. The box holds the
middle half of the values, a line across it is the median, two whiskers reach to the furthest
values that are not far from the box, and each value beyond them is a point. It puts many
groups side by side in little room.

It is a shorthand of the grammar: `w.boxplot(wage, group: industry)` is
`w.plot(x: industry, y: wage).boxplot()`, the same `Plot` value. So everything a plot takes
may follow it in the chain: `coord(flip)` for boxes along the page, which gives long names
their room, `labels`, `theme`, `export`.

`by:` gives each value of a variable a panel of its own, as `facet(by:)` does.

In R this is `boxplot(y ~ g)`, and with ggplot2 `geom_boxplot()`. In Stata it is
`graph box y, over(g)`.

## Arguments
- data: The frame whose rows are drawn.
- var: The variable, or the expression, the boxes are of: a number.
- group: The variable with a box for each of its values. Without it there is one box.
- by: A variable, or several in brackets, whose values part the plot into panels with the
  same axes.

## Value
A `Plot`, the one its grammar form gives. It is drawn where it is printed and may be bound,
extended and exported.

## Details
The five numbers. The line in the box is the median. The box runs from the lower hinge to the
upper: the medians of the lower and of the upper half of the values in order, the middle
value counting in both halves where their number is odd. These are Tukey's hinges, as R's
`boxplot.stats` computes them, close to the quartiles and not always equal to them. With `H`
the height of the box, a whisker runs from each hinge to the furthest value within `1.5 H` of
it, and a value further off is drawn as a point of its own.

Values beyond the whiskers are not errors: in a sample from a normal distribution about seven
in a thousand lie there, and in a skewed one many more. They are the values to look at, not
the values to remove.

A box hides the shape within it, and two peaks look like one; for two or three groups a
`histogram` or a `density` of each says more.

Rows not used. A row with no value of `var`, or of `group:` where one is given, is left out,
and a note says how many there are.

What the shorthand does not take. Boxes filled by a second variable, side by side, and
`outliers: false` are written in the grammar form, whose layer, `boxplot` of a plot, has its
own help page. A refusal of the shorthand is the refusal of `plot`, of the layer or of
`facet`, and points at the argument of `boxplot` that caused it.

## References
Tukey (1977), Exploratory Data Analysis, Addison-Wesley, Reading, Massachusetts.
McGill, Tukey and Larsen (1978), Variations of box plots, The American Statistician 32,
12–16.

## See also
plot, histogram, density, facet, coord

## Examples
> let d = frame(score = [1, 2, 2, 3, 3, 3, 4, 4, 5, 9], arm = ["a", "b", "a", "b", "a", "b", "a", "b", "a", "b"])
> d.boxplot(score, group: arm)
plot · d · score against arm

  axis   title   scale      limits       breaks
  x      arm     category   0.4 to 2.6   a, b
  y      score   linear     0.6 to 9.4   2, 4, 6, 8

  layer   mark      data   marks   settings
      1   boxplot   d          2   1 value beyond the whiskers
