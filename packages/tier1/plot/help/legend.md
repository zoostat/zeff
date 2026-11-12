# legend

## Summary
Where the legends of a plot stand

## Description
A plot has a legend for each variable it shows by colour, fill, size, shape or type of line:
the title of the variable, and a key for each category or for each of a few values, or a bar
of the colours of a continuous scale. Without `legend` they stand at the right of the panel,
one under the other. `p.legend(top)` puts them above the panel and `p.legend(bottom)` under
it, side by side, which leaves the panel the whole width of the page. `p.legend(none)` draws
no legend. It gives the plot back, so it is written in the chain with the marks, before or
after them.

A legend above or under the panel suits a figure of one column, where a legend at the right
takes a third of the width, and a legend of a few short keys. `none` suits a plot whose
colours need no key: one explained in its caption, or one panel of several that share a
legend.

In R with ggplot2 this is `theme(legend.position = "top")`, and `"bottom"`, `"right"` and
`"none"`. In Stata it is `legend(position(12))`, `legend(position(6))` and `legend(off)`.

## Arguments
- p: The plot whose legends are placed.
- position: Where the legends stand, a word without quotation marks: `right`, at the right
  of the panel; `top`, above it; `bottom`, under it and the title of its axis; `none`, not
  drawn. `direct`, the name of each line written at its end, is not built yet.

## Value
The `Plot` with its legends placed. The plot given is not changed. What a later `legend` says
takes the place of what an earlier one said.

## Details
At the right. The legends stand one under the other from the top of the panel, each with its
title above its keys. A legend of more keys than the height holds runs on in a second column,
and a legend that does not fit under the one before it starts a new column to its right.

Above and under the panel. The legends stand side by side from the left edge of the panel,
each with its title above its keys. The keys are laid in rows: in one row where the width of
the page holds them, and in as few rows as it does where it does not. A legend that does not
fit beside the one before it starts a new line under it. The bar of a continuous scale lies
down, its smallest value at the left, with its labels under it. The room the legends take is
taken from the height of the panel, so a legend of many categories on a small page may leave
no room for the panel: the figure is then refused when it is exported, with the room that is
left named, and a larger `size:` or `legend(right)` mends it.

No legend. `none` takes the legends out of the figure and gives their room to the panel. The
scales are as they were: the colours, sizes and shapes of the marks do not change, and the
printed plot still lists them.

What is unchanged. `legend` moves the legends and nothing else: the scales, the keys and
their order are those of the plot. What a legend shows is set by the mappings of the plot and
by `scale`.

The printed plot says where the legends stand, on its last line, when that is not at the
right.

## See also
scale, plot, points, lines, export

## Examples
> let d = frame(dose = [1, 2, 3, 4], change = [0.4, 1.1, 2.3, 2.9], arm = ["a", "b", "a", "b"])
> d.plot(x: dose, y: change, color: arm).points().legend(top)
plot · d · change against dose

  axis   title    scale    limits           breaks
  x      dose     linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  aesthetic   title   scale      values   shown as
  color       arm     category   a, b     orange, sky blue

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1

legend: above the panel
> d.plot(x: dose, y: change, color: arm).points().legend(none)
plot · d · change against dose

  axis   title    scale    limits           breaks
  x      dose     linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  aesthetic   title   scale      values   shown as
  color       arm     category   a, b     orange, sky blue

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1

legend: not drawn
