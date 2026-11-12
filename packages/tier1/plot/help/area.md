# area

## Summary
The area between a line and zero, filled

## Description
Adds a layer of filled areas to a plot: the rows are put in the order of `x` and joined as
`lines` joins them, and the space between that line and zero is filled. An area shows an
amount and how it changes. What the eye reads is the size of the filled space, so an area
suits a quantity for which zero means none: a count, a total, a stock.

One area is drawn for each group, each down to zero by itself. `group:` names the variable
whose values part the rows into areas, and a variable shown by `fill:` parts them too, since an
area has one colour: `fill: union` fills an area for each category of `union`, each in its
colour, with a legend. The areas lie one over another in the order of the groups, and `alpha:`
lets the ones underneath show through.

An area has no outline. `p.area().lines()` draws the line along its upper edge.

`x:`, `y:`, `fill:` and `group:` of `area` take the place, for this layer alone, of what the
plot says of them.

In R with ggplot2 this is `geom_area(position = "identity")`; without that option R puts the
areas of the groups on top of one another. In Stata it is `twoway area y x, sort`.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to `x` and one to `y`.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot. What the layer names itself is read in this frame.
- x: For this layer alone, the variable or the expression its areas are placed by along `x`,
  and ordered by.
- y: The same along `y`: the upper edge of the area, or the lower one where it is below zero.
- fill: For this layer alone, as on `plot`: a bare name fills an area for each category of a
  variable, each in its colour, and a string sets one colour for every area. An area has one
  colour, so a number without value labels, which is shown on a ramp, is refused.
- group: The variable whose values part the rows into areas, for this layer alone, in place of
  `group:` of the plot. Any type will do.
- alpha: The opacity of the fill, from 0, which shows nothing, to 1, which hides what is under
  the area.

## Value
The `Plot` with the layer of areas on top. The plot given is not changed, so a plot bound to a
name can be extended in more than one way.

## Details
Zero. An area reaches from its line to zero, so the `y` axis of a plot with an area always
holds zero. Where `y` is below zero the area hangs down from zero to the line.

The order. Within a group the rows are joined in the order of `x`, and rows with one value of
`x` in the order of the frame, as for `lines`.

Rows not drawn. A row with `NA` or an infinite value of `x` is left out. A row with `NA` or
an infinite value of `y` is left out too, and the area is broken there: it ends at the row
before and begins again at the row after, with an upright edge at each, so a gap in the data
shows as a gap in the area. A note counts the rows left out, and another the breaks. A row
left with no neighbour fills nothing, and a note counts such rows.

Groups. The rows are parted by every variable that `group:` and `fill:` name, together. Where
the plot maps a variable of categories to `color`, and the frame of the layer holds it, the
rows are parted by it as well, so that `plot(x: year, y: wage, color: union).area().lines()`
fills an area under each line; the areas take their colour from `fill`, not from `color`.

The fill. Without `fill:` an area is filled in the dark grey of the theme's marks. With the
usual `alpha:` of 0.6 the gridlines and the areas underneath show through. The key of a legend
is a patch of the fill, as see-through as the areas.

## See also
ribbon, lines, step, plot

## Examples
> let sales = frame(month = [1, 2, 3, 4, 5], units = [12, 18, 15, 21, 24])
> sales.plot(x: month, y: units).area().lines()
plot · sales · units against month

  axis   title   scale    limits         breaks
  x      month   linear   0.8 to 5.2     1, 2, 3, 4, 5
  y      units   linear   -1.2 to 25.2   0, 5, 10, 15, 20, 25

  layer   mark    data    marks   settings
      1   area    sales       5   alpha 0.6, 1 area
      2   lines   sales       5   width 1, 1 line
> let shops = frame(month = [1, 2, 3, 1, 2, 3], units = [12, 18, 15, 7, 9, 14], shop = ["north", "north", "north", "south", "south", "south"])
> shops.plot(x: month, y: units, fill: shop).area(alpha: 0.4)
plot · shops · units against month

  axis   title   scale    limits         breaks
  x      month   linear   0.9 to 3.1     1, 1.5, 2, 2.5, 3
  y      units   linear   -0.9 to 18.9   0, 5, 10, 15

  aesthetic   title   scale      values         shown as
  fill        shop    category   north, south   orange, sky blue

  layer   mark   data    marks   settings
      1   area   shops       6   alpha 0.4, 2 areas
