# theme

## Summary
The theme a figure is drawn by: its type, its lines and its spacing

## Description
A theme is everything of a figure that is not its data: the size of the type, the lines of
the axes and their ticks, the gridlines, the margins and the gaps. Without `theme` a figure is
drawn by `journal`, which is made for a figure in an article: a white page, thin dark axes
with ticks, light gridlines at the breaks, and type from 8 points. `p.theme(minimal)` leaves
out the lines of the axes and their ticks. `p.theme(slide)` draws everything twice as large,
for a figure shown on a screen. `base_size:` sets the size of the type, and every other size
with it. It gives the plot back, so it is written in the chain with the marks.

Every theme has a white page, gridlines at the breaks of the axes and nowhere else, and
nothing that is there for ornament.

In R with ggplot2 this is `theme_bw()` or `theme_minimal()` with `base_size =`. In Stata it
is `scheme()`.

## Arguments
- p: The plot whose theme is set.
- name: The theme, a word without quotation marks: `journal`, `minimal` or `slide`.
- base_size: The size of the titles of the axes, in points of 1/72 inch, 9 or more. Every
  other size of the theme is a fixed share of it. Left out, it is 9, and 18 with `slide`.

## Value
The `Plot` with its theme set. The plot given is not changed. What a later `theme` says takes
the place of all that an earlier one said.

## Details
The sizes. At the base size of 9 points the titles of the axes and of the legends are 9
points, the labels of the breaks and of the keys 8, the title of the figure 10 and its
caption 8; the lines of the axes are half a point wide and the gridlines a third. A base size
of 12 makes each of these a third larger, and the margins and gaps with them. The labels are
eight ninths of the base size, and no text of a figure is set below 8 points, so a base size
below 9 is refused.

`journal`. The axes are drawn as lines at the left and the foot of the panel, with a tick at
each break.

`minimal`. No line of an axis and no tick is drawn: the gridlines carry the eye from a label
to the marks. The labels stand a tick nearer the panel. Everything else is as in `journal`.

`slide`. As `journal` at a base size of 18 points, for a figure of the size `slide` of
`export`, 254 by 143 mm, which is shown on a screen. On a figure of one column so large a
type leaves little room for the panel.

What a theme does not change. The scales, the colours and the marks are those of the plot.
The size of a point and the width of a line are given in points by the marks, `size:` and
`width:`, and are not made larger by a larger base size: on a slide, give the marks a size to
match, as in `points(size: 9)`.

The printed plot says the theme on its last line, when it is not `journal` at its own size.
The theme is used when the figure is written by `export`.

## See also
plot, labels, legend, export

## Examples
> let d = frame(dose = [1, 2, 3, 4], change = [0.4, 1.1, 2.3, 2.9])
> d.plot(x: dose, y: change).points().theme(minimal)
plot · d · change against dose

  axis   title    scale    limits           breaks
  x      dose     linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1

theme: minimal
> d.plot(x: dose, y: change).points().theme(slide, base_size: 24)
plot · d · change against dose

  axis   title    scale    limits           breaks
  x      dose     linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1

theme: slide, base size 24
