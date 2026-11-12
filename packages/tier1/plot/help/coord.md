# coord

## Summary
The coordinates of a plot: which way its two axes run on the page

## Description
The coordinates say how the two positions of a plot, `x` and `y`, are laid on the page.
Without `coord` they are `cartesian`: `x` runs along the foot of the panel and `y` up its
side. `p.coord(flip)` exchanges the two: `x` runs up the side and `y` along the foot, and
every mark is turned with them. Bars lie along the page, a line runs upwards, and a reference
line across the panel becomes one down it. It gives the plot back, so it is written in the
chain with the marks, before or after them.

Flipped coordinates suit a plot whose `x` holds many categories, or categories with long
names. Beside lying bars every name is written in full on a line of its own; under standing
bars the names run into one another and some are left out.

In R with ggplot2 this is `coord_flip()`. In Stata it is the option `horizontal` of a graph,
or `graph hbar` in the place of `graph bar`.

## Arguments
- p: The plot whose coordinates are set.
- type: The coordinates, a word without quotation marks: `cartesian`, `x` along the foot and
  `y` up the side; or `flip`, the two exchanged. Left out, it is `cartesian`. The words
  `polar` and `fixed` are kept for coordinates that are not built yet, and are refused.
- ratio: The length on the page of one unit of `y`, in units of `x`, for `fixed`. It is not
  built yet, and is refused.

## Value
The `Plot` with its coordinates set. The plot given is not changed. What a later `coord` says
takes the place of what an earlier one said: `flip` written twice is still flipped.

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

The printed plot says the flipped coordinates on a line of its own under the table of layers.
Its table of axes is that of the data, with `x` and `y` as the script names them.

## See also
plot, bars, scale_x, labels, legend

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
