# coord

## Summary
The coordinates of a plot: which way the axes run and how long their units are

## Description
The coordinates say how the two positions of a plot, `x` and `y`, are laid on the page.
Without `coord` they are `cartesian`: `x` runs along the foot of the panel and `y` up its
side. `p.coord(flip)` exchanges the two: `x` runs up the side and `y` along the foot, and
every mark is turned with them. Bars lie along the page, a line runs upwards, and a reference
line across the panel becomes one down it. It gives the plot back, so it is written in the
chain with the marks, before or after them.

`p.coord(fixed)` keeps `x` along the foot and `y` up the side, and gives a unit of `y` the
same length on the page as a unit of `x`: the panel takes the shape of the data, where
otherwise it takes the shape of the page. `ratio:` makes a unit of `y` as long as that many
units of `x`.

Flipped coordinates suit a plot whose `x` holds many categories, or categories with long
names. Beside lying bars every name is written in full on a line of its own; under standing
bars the names run into one another and some are left out. Fixed coordinates suit a plot
whose two axes are in the same unit, where a slope or a distance is to be read off the page:
a map, one measurement against another of the same kind, what was observed against what was
fitted. On such a plot a square of the data is a square, and a line at 45 degrees is the line
of equal values.

In R with ggplot2 these are `coord_flip()` and `coord_fixed(ratio = )`. In Stata the first is
the option `horizontal` of a graph, or `graph hbar` in the place of `graph bar`; the nearest
to the second is `aspectratio()`, which gives the shape of the plot region and not the length
of a unit.

## Arguments
- p: The plot whose coordinates are set.
- type: The coordinates, a word without quotation marks: `cartesian`, `x` along the foot and
  `y` up the side; `flip`, the two exchanged; or `fixed`, as `cartesian` with a unit of each
  axis in a fixed proportion. Left out, it is `cartesian`. The word `polar` is kept for
  coordinates that are not built yet, and is refused.
- ratio: With `fixed`, how many units of `x` are as long on the page as one unit of `y`, a
  number above 0. Left out, it is 1. Above 1 it makes a unit of `y` longer than a unit of
  `x`, and below 1 shorter.

## Value
The `Plot` with its coordinates set. The plot given is not changed. What a later `coord` says
takes the place of what an earlier one said: `flip` written twice is still flipped, and
`fixed` after `flip` is fixed and upright.

## Details
What is not changed. A flipped plot is the same plot on another page: `x` is still `x`. The
mappings, the scales, their limits and their breaks are those of the upright plot.
`scale_x` still sets the scale of `x`, and `labels(x:)` its title, wherever the axis is
drawn. Each mark reads its positions as before: bars still start from zero along `y`,
`hline(at:)` is still a line at one value of `y`, which on the flipped page runs down the
panel, and `vline(at:)` a line at one value of `x`, which runs across it. So a plot can be
built and checked upright, and `coord(flip)` added as the last word.

What is turned. Every mark is drawn where the upright plot has it, with its distance along
the foot and its distance up the side exchanged. The first category, and the smallest number,
of `x` is at the foot of the side, as the smallest number of `y` is in an upright plot. Text
is not turned: the labels of `text` and of `annotate` stay upright, each at the place of its
row. The sizes of points and the widths of lines are as they were.

The axes. The axis of `x` is drawn at the left with each label written across the page, on
one line, and its title turned up the side. The axis of `y` is drawn at the foot. The labels
of categories along the foot of a flipped plot, those of `y`, are broken over lines or
thinned as those of `x` are in an upright one. Labels at the left are never broken: where
they are so long that no panel is left beside them, the figure is refused and the message
says how much room there was.

The legends. They stand where `legend` puts them. `legend(direct)` writes the name of each
line at its right-hand end; in a flipped plot the lines end at the top, so the legend is
drawn at the right of the panel in its place.

Fixed coordinates: the shape of the panel. The limits of the two axes say how many units each
runs over, and the ratio how long a unit of `y` is against a unit of `x`; the two together
give the panel its shape. If `x` runs over 30 units and `y` over 20, the panel is two thirds
as high as it is wide at a ratio of 1, and as high as it is wide at a ratio of 1.5. The panel
is the largest of that shape for which the page has room, and it stands in the middle of that
room with its axes, their titles and a legend beside it; the title of the figure and its
caption stay at the edges of the page. The scales, the limits and the breaks are not changed.

Fixed coordinates: what a unit is. On an axis of numbers it is one. On an axis of categories
it is the place of one category. On a log axis it is a power of ten, so that a slope of one
on the page is an elasticity of one, and on a square-root axis one of the square root. On an
axis of dates it is a day.

Fixed coordinates: a panel too thin. Two axes that do not share a unit, a wage against years
of schooling, have no ratio that means anything, and at a ratio of 1 the panel may be a sliver.
A figure whose panel would be less than 12.7 mm wide or high is refused when it is written by
`export`, and the message names the ratio at which the panel fills the page.

The printed plot says flipped or fixed coordinates on a line of its own under the table of
layers. Its table of axes is that of the data, with `x` and `y` as the script names them.

## See also
plot, bars, scale_x, labels, legend, export

## Examples
> let d = frame(arm = ["placebo", "low dose", "high dose"], change = [0.4, 1.1, 2.3])
> d.plot(x: arm, y: change).bars().coord(flip)
plot · d · change against arm

  axis   title    scale      limits            breaks
  x      arm      category   0.4 to 3.6        high dose, low dose, placebo
  y      change   linear     -0.115 to 2.415   0, 0.5, 1, 1.5, 2

  layer   mark   data   marks   settings
      1   bars   d          3   width 0.9, alpha 1

coordinates: flipped, `x` up the side and `y` along the foot
> d.plot(x: arm, y: change).bars().hline(at: 1).coord(flip).labels(y: "Change in score")
plot · d · change against arm

  axis   title             scale      limits            breaks
  x      arm               category   0.4 to 3.6        high dose, low dose, placebo
  y      Change in score   linear     -0.115 to 2.415   0, 0.5, 1, 1.5, 2

  layer   mark    data   marks   settings
      1   bars    d          3   width 0.9, alpha 1
      2   hline   d          1   at 1, width 0.5, style dashed

coordinates: flipped, `x` up the side and `y` along the foot
> let s = frame(x = [0, 3, 3, 0, 0], y = [0, 0, 2, 2, 0])
> s.plot(x: x, y: y).lines().coord(fixed)
plot · s · y against x

  axis   title   scale    limits          breaks
  x      x       linear   -0.15 to 3.15   0, 0.5, 1, 1.5, 2, 2.5, 3
  y      y       linear   -0.1 to 2.1     0, 0.5, 1, 1.5, 2

  layer   mark    data   marks   settings
      1   lines   s          5   width 1, 1 line

coordinates: fixed, one unit of `y` as long as one of `x`
> s.plot(x: x, y: y).lines().coord(fixed, ratio: 1.5)
plot · s · y against x

  axis   title   scale    limits          breaks
  x      x       linear   -0.15 to 3.15   0, 0.5, 1, 1.5, 2, 2.5, 3
  y      y       linear   -0.1 to 2.1     0, 0.5, 1, 1.5, 2

  layer   mark    data   marks   settings
      1   lines   s          5   width 1, 1 line

coordinates: fixed, one unit of `y` as long as 1.5 of `x`
