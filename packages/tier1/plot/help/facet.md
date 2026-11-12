# facet

## Summary
A plot parted into panels, one for each value of a variable

## Description
Small multiples: the same plot drawn once for each value of a variable, the panels side by
side on the same scales. `p.facet(by: industry)` gives a panel for each industry, with the
name of the industry written over it, and in each panel the marks of the rows of that
industry alone. It gives the plot back, so it is written in the chain with the marks, before
or after them.

Panels are for comparing groups that would hide one another in one panel: twelve clouds of
points in twelve colours are a blur, and twelve small clouds side by side can be read. The
panels share their axes, so the same place on the page is the same value in every panel, and
a difference between two panels is a difference in the data.

In R with ggplot2 this is `facet_wrap(~ industry)`. In Stata it is the option
`by(industry)` of a graph.

## Arguments
- p: The plot that is parted.
- by: The variable whose values part the rows, a bare name, or several in square brackets:
  `by: [industry, union]` has a panel for each pair of values that the frame holds.
- rows: Kept for a grid of panels with a variable down the page. It is not built yet, and
  is refused.
- cols: Kept for a grid of panels with a variable across the page. It is not built yet, and
  is refused.
- scales: Whether the panels share their axes, a word without quotation marks. `fixed`, the
  default, is the one way built so far: every panel has the same two axes. `free`, `free_x`
  and `free_y` are refused.

## Value
The `Plot` parted into panels. The plot given is not changed. What a later `facet` says
takes the place of what an earlier one said.

## Details
The panels. There is one for each value of the variable that the frame of the plot holds, in
the order an axis or a legend would have them in: the levels of a Category in their order,
and other values from the smallest up. Over each is written its value, by its value label
where the variable has one. A row with no value of the variable is not dropped: it is drawn
in a panel of its own, the last, named `NA`. A value that only a layer with `data:` of its
own holds is a panel more, after the others.

The marks. Each panel is the plot of its own rows. Bars are stacked, or set side by side,
among the bars of their panel; a line joins the rows of its panel and never crosses to the
next; stacked areas are stacked panel by panel. A layer that does not have the variable is
drawn whole in every panel: a reference line at a number, an annotation, a layer whose own
frame lacks it. That is how one line or one cloud of all the rows is put behind every panel
for comparison.

The scales. The axes, the colours, the sizes and the shapes are those of the plot without
facets, trained over all the rows, and every panel is drawn by them. The width of a bar is
the same in every panel.

The page. The panels are of one size and are laid in rows from the top left: one row for up
to three panels, two for up to six, three for up to twelve, and beyond that about as many
rows as columns. The labels of `y` are written beside the first panel of each row and those
of `x` under the lowest panel of each column; each axis has its title once, and the legends
stand beside the whole. Where that grid would leave each panel less than 12.7 mm wide or
high on the page of the figure, the panels are laid in more columns or fewer, so that forty
panels on a page of two columns are four rows of ten. Where no grid gives the panels their
room, the figure is refused when it is written by `export`: give a larger `size:`.

With the other functions. `coord(flip)`, `coord(fixed)` and `coord(polar)` are applied in
every panel. `legend(direct)` writes names at the ends of lines beside one panel, so with
facets the legend is drawn at the right. `labels`, `theme` and the scales are those of the
whole plot.

The printed plot says the panels on a line of its own under the table of layers: how many
they are, the variable, and what is written over them. Its table of layers counts the marks
of each layer over all the panels.

## See also
plot, coord, legend, scale, export

## Examples
> let d = frame(site = ["north", "south", "north", "south"], week = [1, 1, 2, 2], count = [4, 7, 5, 9])
> d.plot(x: week, y: count).lines().points().facet(by: site)
plot · d · count against week

  axis   title   scale    limits         breaks
  x      week    linear   0.95 to 2.05   1, 1.2, 1.4, 1.6, 1.8, 2
  y      count   linear   3.75 to 9.25   4, 5, 6, 7, 8, 9

  layer   mark     data   marks   settings
      1   lines    d          4   width 1, 2 lines
      2   points   d          4   size 4.5, alpha 1

panels: 2 by site: north, south
> d.plot(x: week, y: count).bars().hline(at: 6).facet(by: site)
plot · d · count against week

  axis   title   scale    limits           breaks
  x      week    linear   0.455 to 2.545   0.5, 1, 1.5, 2, 2.5
  y      count   linear   -0.45 to 9.45    0, 2, 4, 6, 8

  layer   mark    data   marks   settings
      1   bars    d          4   width 0.9, alpha 1
      2   hline   d          2   at 6, width 0.5, style dashed

panels: 2 by site: north, south
