# bars

## Summary
A bar from zero at each value of x: as high as y, or as the count of rows

## Description
Adds a layer of bars to a plot. A bar stands at a value of `x` and reaches from zero to an
amount, so the eye compares the lengths of the bars: a bar chart shows how much, or how many,
for each of a few categories or values.

What the amount is depends on `y`. Where the plot or the layer maps something to `y`, each
row is a bar as high as its `y`: `m.plot(x: industry, y: wage).bars()` draws a mean wage that
was computed before. Where nothing is mapped to `y`, the bars count: there is one bar for
each value of `x`, as high as the number of rows that have it, so `w.plot(x: industry).bars()`
shows how many rows each industry has, and the `y` axis is titled `count`.

`fill:` colours the bars by the categories of a variable, with a legend, and `group:` parts
the rows without a colour. `position:` says what happens to the bars of several groups at one
`x`: `stack` puts them on one another, `dodge` side by side, and `fill` stacks them as shares
of the total, from 0 to 1.

`x` may be categories, a Str or a Category, or numbers. `y` must be numbers.

In R with ggplot2 this is `geom_col()` where `y` is mapped and `geom_bar()` where it is not,
with the same three positions. In Stata it is `graph bar (asis) y, over(x)` for values and
`graph bar (count), over(x)` for counts, with `stack` and `percentages` among its options.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to `x`.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot. What the layer names itself is read in this frame.
- x: For this layer alone, the variable or the expression at whose values the bars stand.
- y: For this layer alone, the variable or the expression that gives the height of a bar. A
  layer on a plot that maps nothing to `y`, and names none, counts rows.
- fill: For this layer alone, as on `plot`: a bare name fills the bars of each category of a
  variable in its colour, and a string sets one colour for every bar. A bar has one colour,
  so a number without value labels, which is shown on a ramp, is refused.
- group: The variable whose values part the rows into sets of bars without colouring them,
  for this layer alone, in place of `group:` of the plot. Any type will do.
- position: What the bars that share a value of `x` do. `stack`: on one another. `dodge`:
  side by side. `fill`: on one another, as shares of the total at that `x`.
- width: The width of a bar, as a share of the gap between two neighbouring values of `x`,
  above 0 and at most 1. On an axis of categories the gap is one place.
- alpha: The opacity of the bars, from 0, which shows nothing, to 1, which is solid.

## Value
The `Plot` with the layer of bars on top. The plot given is not changed, so a plot bound to a
name can be extended in more than one way.

## Details
Zero. A bar starts at zero, always, because its length is what is read: a bar that starts
elsewhere shows a difference as if it were a ratio. The `y` axis of a plot with bars therefore
holds zero. A value below zero hangs down from zero.

Values. With `y` mapped, every row is a bar. Several rows with one `x` and one group are
several bars, stacked on one another or, with `dodge`, drawn in one place; where one bar for
each is wanted, summarise first, as in `w.aggregate(wage = mean(wage), by: industry)`.

Counts. Without `y`, a row counts once at its `x`, whatever else it holds. A row with `NA` in
`x` is not counted, and a note says how many. A category of an axis that has no row has no
bar, and keeps its place.

Groups. The rows are parted by every variable that `group:` and `fill:` name, together, and
by the categories of a variable the plot maps to `color`, where the frame of the layer holds
it. The groups come in the order of their categories, `NA` last.

Stacked bars. The first group is on top, so that the bars read from top to bottom as the keys
of the legend do. Values below zero are stacked downwards from zero, apart from the values
above it. With `fill` each bar is divided by the total at its `x`; an `x` with a value below
zero, or a total that is not above zero, has no shares to show and is left out, and a note
counts its bars.

Bars side by side. With `dodge` the bars at one `x` share the width of one bar equally, the
first group at the left. Where a group has no row at an `x`, the others there are wider:
this is what R does, and the rows can be completed first where bars of one width are wanted.

Width. On numbers, the gap is the smallest distance between two values of `x` in the layer,
so bars at 1, 2 and 4 are 0.9 wide and do not touch. The `x` axis is made wide enough for the
whole of the first and the last bar.

The fill. Where neither the plot nor the layer says anything of `fill`, a bar takes its fill
from `color:` of the plot, so `plot(x: industry, y: wage, color: female).bars(position: dodge)`
draws a bar for each sex in its colour, with a legend. A colour by a number, which is shown on
a ramp, fills nothing, since a bar has one fill. `fill:` on the plot or on the layer comes
first. With nothing to take a fill from, a bar is filled in the dark grey of the theme's
marks. A bar has no outline. The key of a legend is a patch of the fill.

## See also
plot, area, points, aggregate

## Examples
> let sales = frame(shop = ["north", "south", "east", "north", "south"], units = [12, 18, 15, 7, 9], kind = ["s", "s", "s", "l", "l"])
> sales.plot(x: shop).bars()
plot · sales · shop

  axis   title   scale      limits        breaks
  x      shop    category   0.4 to 3.6    east, north, south
  y      count   linear     -0.1 to 2.1   0, 0.5, 1, 1.5, 2

  layer   mark   data    marks   settings
      1   bars   sales       3   width 0.9, alpha 1
> sales.plot(x: shop, y: units).bars()
plot · sales · units against shop

  axis   title   scale      limits           breaks
  x      shop    category   0.4 to 3.6       east, north, south
  y      units   linear     -1.35 to 28.35   0, 5, 10, 15, 20, 25

  layer   mark   data    marks   settings
      1   bars   sales       5   width 0.9, alpha 1
> sales.plot(x: shop, y: units, fill: kind).bars(position: dodge)
plot · sales · units against shop

  axis   title   scale      limits         breaks
  x      shop    category   0.4 to 3.6     east, north, south
  y      units   linear     -0.9 to 18.9   0, 5, 10, 15

  aesthetic   title   scale      values   shown as
  fill        kind    category   l, s     orange, sky blue

  layer   mark   data    marks   settings
      1   bars   sales       5   width 0.9, position dodge, alpha 1
