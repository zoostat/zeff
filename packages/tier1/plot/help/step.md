# step

## Summary
Step lines through the rows, in the order of x

## Description
Adds a layer of step lines to a plot: the rows are put in the order of `x` and joined as
`lines` joins them, but by level and upright pieces, like stairs, and never at a slant. A step
line shows a quantity that holds its value until it changes: a price, a count, a rate set at
dates, an empirical distribution function.

`p.step()` goes level from each row to the `x` of the next and then up or down to it, so the
value of a row holds until the next row. `direction:` chooses the other ways.

Everything else is as for `lines`: one line for each group, the rows parted by `group:` and by
the categories that `color:` and `linetype:` show; `width:` for the width of the lines; and
`x:`, `y:`, `color:` and `group:` of the layer in the place of those of the plot.

In R with ggplot2 this is `geom_step()`, with the same three directions. In Stata it is
`line y x, sort connect(stairstep)`, which goes level first, and `connect(stepstair)`, which
goes upright first.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to `x` and one to `y`.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot. What the layer names itself is read in this frame.
- x: For this layer alone, the variable or the expression its lines are placed by along `x`,
  and ordered by.
- y: The same along `y`.
- color: For this layer alone, as on `plot`: a bare name draws a line for each category of a
  variable, each in its colour, and a string sets one colour for every line.
- group: The variable whose values part the rows into lines, for this layer alone, in place of
  `group:` of the plot.
- linetype: A bare name draws a line for each category of a variable, each of its own type; a
  string sets one type for every line, as `"dashed"`.
- width: The width of a line, in points of 1/72 inch, above 0.
- direction: How the line goes from one row to the next. `hv`: level to the `x` of the next
  row, then up or down. `vh`: up or down to the `y` of the next row, then level. `mid`: level
  to halfway between the two rows, up or down there, and level again.

## Value
The `Plot` with the layer of step lines on top. The plot given is not changed.

## Details
The line passes through every row it draws, whatever the direction. With `hv` the value of a
row is shown from its own `x` to the next, which is right for a value that holds until it
changes. With `vh` it is shown from the `x` before it to its own, which is right for a total
over the interval that ends at the row. With `mid` each row stands in the middle of its level
piece, which is right for a value that belongs to a place and not to an interval.

The rows, the groups, the rows not drawn and the types of line are those of `lines`: a row
with `NA` in `y` breaks the line there, a row left with no neighbour draws nothing, and notes
count both. Two rows with one `y` are joined by one level piece.

## See also
lines, plot, points

## Examples
> let rate = frame(month = [1, 4, 6, 9], percent = [5.0, 5.25, 5.0, 4.5])
> rate.plot(x: month, y: percent).points().step()
plot · rate · percent against month

  axis   title     scale    limits             breaks
  x      month     linear   0.6 to 9.4         2, 4, 6, 8
  y      percent   linear   4.4625 to 5.2875   4.6, 4.8, 5, 5.2

  layer   mark     data   marks   settings
      1   points   rate       4   size 4.5, alpha 1
      2   step     rate       4   width 1, 1 line
> rate.plot(x: month, y: percent).step(direction: mid, linetype: "dashed")
plot · rate · percent against month

  axis   title     scale    limits             breaks
  x      month     linear   0.6 to 9.4         2, 4, 6, 8
  y      percent   linear   4.4625 to 5.2875   4.6, 4.8, 5, 5.2

  layer   mark   data   marks   settings
      1   step   rate       4   width 1, direction mid, linetype dashed, 1 line
