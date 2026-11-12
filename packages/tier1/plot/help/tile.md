# tile

## Summary
A filled rectangle at each row's x and y: heat maps

## Description
Adds a layer of tiles to a plot: for each row a rectangle centred on its `x` and its `y`, as
wide and as high as the gap between two neighbouring places, so that the tiles of a full grid
touch and cover the panel. The colour of a tile shows a third variable. A layer of tiles is a
heat map: a table of numbers read by colour, such as a correlation for each pair of
variables, a mean for each month of each year, or a count for each cell of a cross-tabulation.

`fill:` names the variable shown. A number without value labels is shown on a continuous
scale of colours, from dark purple for the smallest value to yellow for the largest, with a
bar of the colours for a legend. A variable of categories gives each category a colour of its
own, with a key for each.

`x` and `y` may each be categories, a Str or a Category, or numbers. On an axis of categories
a tile is one place wide. On numbers it is as wide as the smallest distance between two
values, so the tiles of a regular grid touch.

A tile is one row. Where a frame has several rows for one cell, summarise them first with
`aggregate`, or the tiles of the cell are drawn on one another and the last is seen.

In R with ggplot2 this is `geom_tile()`. In Stata it is `heatplot`, a command of the user
community, or `twoway contour` with the option `heatmap`.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to `x` and one to `y`.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot. What the layer names itself is read in this frame.
- x: For this layer alone, the variable or the expression the tiles are placed by along `x`.
- y: The same along `y`.
- fill: For this layer alone, as on `plot`: a bare name fills each tile by the value of a
  variable in its row, and a string sets one colour for every tile.
- alpha: The opacity of the tiles, from 0, which shows nothing, to 1, which is solid.

## Value
The `Plot` with the layer of tiles on top. The plot given is not changed, so a plot bound to a
name can be extended in more than one way.

## Details
The size of a tile. The gap is taken along each axis by itself: one place on an axis of
categories, and on numbers the smallest distance between two values of the rows drawn. Every
tile of a layer has that size, so a grid with a missing cell shows the paper there, and an
irregular grid shows gaps between its wider steps. The axes are made wide enough for the
whole of the outermost tiles.

Rows not drawn. A row with `NA` or an infinite value in `x` or `y` is not drawn, and neither
is a row without the number its fill shows on a continuous scale; a note counts them. A row
without a category of its fill is drawn, in the grey of `NA`.

The fill. Where neither the plot nor the layer says anything of `fill`, a tile takes its fill
from `color:` of the plot, a variable or one colour. With nothing to take a fill from, every
tile is filled in the dark grey of the theme's marks, which shows where the rows are and
nothing more. A tile has no outline.

The continuous scale runs exactly from the smallest value drawn to the largest. It is
viridis, whose colours grow lighter from one end to the other, so a heat map still reads in
grey. The legend is a bar of the colours with the values beside it.

## See also
plot, bars, points, aggregate

## Examples
> let g = frame(month = [1, 2, 3, 1, 2, 3], year = ["2025", "2025", "2025", "2026", "2026", "2026"], sales = [4, 7, 9, 5, 8, 12])
> g.plot(x: month, y: year, fill: sales).tile()
plot · g · year against month

  axis   title   scale      limits         breaks
  x      month   linear     0.35 to 3.65   0.5, 1, 1.5, 2, 2.5, 3, 3.5
  y      year    category   0.4 to 2.6     2025, 2026

  aesthetic   title   scale    values    shown as
  fill        sales   linear   4 to 12   viridis, dark purple to yellow

  layer   mark   data   marks   settings
      1   tile   g          6   alpha 1
