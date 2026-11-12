# scale

## Summary
The colours, sizes and shapes a plot shows its values as

## Description
Says what the scales that are not positions show their values as: the colours of `color:` and
of `fill:`, the sizes of `size:` and the shapes of `shape:`. Without it a plot colours
categories with the eight colours of Okabe and Ito, which people with the common forms of
colour blindness can tell apart, and shows a number on viridis, from dark purple to yellow;
a size shows an amount by the area of its mark; and categories take eight shapes in a fixed
order. `p.scale(...)` changes any of these. It gives the plot back, so it is written in the
chain with the marks.

`color: ["grey", "firebrick"]` gives the first category the first colour and the second the
second. For a number, the list is the colours its scale passes through, from the lowest value
to the highest: `color: ["white", "steelblue"]`. `fill:` does the same for the scale of fills.

`size: [2, 12]` draws the smallest value 2 points across and the largest 12. `size: [24]`
keeps the usual reading, by area from zero, with the largest mark 24 points across.

`shape: ["circle_open", "cross"]` gives each category its shape.

The axes have functions of their own, `scale_x` and `scale_y`.

In R with ggplot2 these are `scale_colour_manual()` and `scale_fill_manual()` for categories,
`scale_colour_gradientn()` for a number, `scale_size()` and `scale_size_area()`, and
`scale_shape_manual()`. In Stata they are the options `mcolor()`, `msize()` and `msymbol()`
of each plot.

## Arguments
- p: The plot whose scales are set.
- color: The colours of the scale of `color:`. A list of colours, each a string: a name of
  CSS in small letters, such as `"steelblue"`, or `#` and six hexadecimal digits. Or one of
  two palettes by its name, without quotation marks: `okabe_ito` or `viridis`.
- fill: The colours of the scale of `fill:`, in the same forms.
- size: A list of one number or of two, the diameters of marks in points of 1/72 inch,
  above 0. One number is the largest mark, a size showing an amount by its area. Two are the
  marks of the smallest and of the largest value drawn.
- shape: A list of the names of shapes, each a string, one for each category in their order.

## Value
The `Plot` with its scales set. The plot given is not changed. What a later `scale` says of
one of the four takes the place of what an earlier one said of it, and leaves the other three
as they were.

## Details
Colours for categories. The colours are given to the categories in their order, which is the
order of the legend: the levels of a Category, the order of text for a Str. With fewer colours
than categories the colours are used again from the first, and a note says so; a list longer
than the categories is used as far as it is needed. A row with `NA` keeps the grey of the
category `NA`. `okabe_ito` is the usual palette, said outright. `viridis` gives the
categories colours at equal steps along viridis, which suits categories that have an order,
such as the groups of an age or of a dose, and has a colour for each however many there are.

Colours for a number. A list of two colours or more is a ramp: the first colour stands at the
lowest value drawn, the last at the highest, and the others at equal distances between them.
A colour between two of them is a mixture of the two, in proportion to its place. `viridis`
is the usual ramp, said outright. `okabe_ito` is refused for a number, since its eight
colours have no order, and so is a list of one colour. Choose colours that grow steadily
lighter or darker, so that a reader can tell more from less in print and with any vision; a
ramp through many hues, a rainbow, cannot be read that way.

Fills. A bar, an area and a tile are filled by the scale of `fill:`. Where a plot colours by
`color:` and says nothing of `fill`, such marks take their fill from the colour, and
`scale(color: ...)` sets those fills as well.

Sizes. Without `scale`, and with one number, a size shows an amount: the area of a mark is in
proportion to its value, a value of 0 has no size, and a value below 0 is not drawn. That
reading is honest about amounts, since twice the value is twice the ink. With two numbers a
size shows a difference: the smallest value drawn has the first diameter and the largest the
second, with the areas between in proportion, so that differences are seen whatever the level
of the variable, a year or a temperature. A value of 0 or below then has its size like any
other. The second reading does not show amounts: a value twice another may be drawn many
times as large.

Shapes. The names are those a string given to `shape:` takes: `circle`, `triangle`, `square`,
`diamond`, `triangle_down`, `star`, `plus`, `cross` and `asterisk`, and the outlines
`circle_open`, `triangle_open`, `square_open`, `diamond_open`, `triangle_down_open` and
`star_open`. With fewer shapes than categories they are used again from the first, with a
note. A row with `NA` keeps the asterisk.

Before or after the layers. `scale` may be written before the layer that maps the variable,
and then waits for it, or after it. The notes are raised call by call, so a note of a layer
written before `scale` is printed though `scale` may undo its cause: the categories beyond
the eight colours, to which `scale` then gives colours of their own, or the rows with a size
below 0, which two sizes then draw. Write `scale` before the layers and the notes are those
of the plot as it ends.

## See also
scale_x, scale_y, plot, points, bars

## Examples
> let d = frame(dose = [1, 2, 3, 4], change = [0.4, 1.1, 2.3, 2.9], arm = ["a", "b", "a", "b"])
> d.plot(x: dose, y: change, color: arm).points().scale(color: ["grey", "firebrick"])
plot · d · change against dose

  axis   title    scale    limits           breaks
  x      dose     linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  aesthetic   title   scale      values   shown as
  color       arm     category   a, b     grey, firebrick

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1
> d.plot(x: dose, y: change, color: change).points().scale(color: ["white", "steelblue"])
plot · d · change against dose

  axis   title    scale    limits           breaks
  x      dose     linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  aesthetic   title    scale    values       shown as
  color       change   linear   0.4 to 2.9   a ramp through white, steelblue

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1
> d.plot(x: dose, y: change, size: change, shape: arm).points().scale(size: [2, 12], shape: ["circle_open", "cross"])
plot · d · change against dose

  axis   title    scale    limits           breaks
  x      dose     linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  aesthetic   title    scale      values       shown as
  size        change   sqrt       0.4 to 2.9   from 2 to 12 across
  shape       arm      category   a, b         circle_open, cross

  layer   mark     data   marks   settings
      1   points   d          4   size by change, shape by arm, alpha 1
