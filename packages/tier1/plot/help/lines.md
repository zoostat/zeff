# lines

## Summary
Lines through the rows, in the order of x

## Description
Adds a layer of lines to a plot: the rows are put in the order of `x` and each is joined to
the next, so `m.plot(x: year, y: wage).lines()` draws the wage from year to year. The order of
the rows in the frame does not matter.

One line is drawn for each group. `group:` names the variable whose values part the rows into
lines, as in `group: worker` for a line for each worker. A variable shown by `color:` or by
`linetype:` parts them too, since a line has one colour and one type from end to end:
`color: union` draws a line for each category of `union`, each in its colour, with a legend.

`linetype:` shows categories by the rhythm of the line: whole, dashed, dotted and so on, in a
fixed order of six that can be told apart in black on white, so a figure reads without its
colours. A string sets one type for every line of the layer, as in `linetype: "dashed"`.
`width:` is the width of the lines.

`x:`, `y:`, `color:` and `group:` of `lines` take the place, for this layer alone, of what the
plot says of them, so `p.points().lines(y: fitted)` draws a second variable as a line over the
points of the first.

In R with ggplot2 this is `geom_line()`, which also joins the rows in the order of `x`; a line
in the order of the rows is its `geom_path()`. In Stata it is `line y x, sort`, or
`twoway line`.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to `x` and one to `y`.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot. What the layer names itself is read in this frame.
- x: For this layer alone, the variable or the expression its lines are placed by along `x`,
  and ordered by.
- y: The same along `y`.
- color: For this layer alone, as on `plot`: a bare name draws a line for each category of a
  variable, each in its colour, and a string sets one colour for every line. A line has one
  colour, so a number without value labels, which is shown on a ramp, is refused.
- group: The variable whose values part the rows into lines, for this layer alone, in place of
  `group:` of the plot. Any type will do.
- linetype: A bare name draws a line for each category of a variable, each of its own type; a
  string sets one type for every line: `"solid"`, `"dashed"`, `"dotted"`, `"dash_dot"`,
  `"long_dash"` or `"dash_dot_dot"`.
- width: The width of a line, in points of 1/72 inch, above 0.

## Value
The `Plot` with the layer of lines on top. The plot given is not changed, so a plot bound to a
name can be extended in more than one way.

## Details
The order. Within a group the rows are joined in the order of `x`, from the smallest to the
largest, and rows with one value of `x` in the order of the frame. With several rows at one
`x` in a group the line goes up and down at that `x`; a mean for each `x`, computed first with
`aggregate`, gives one row for each and a line that reads from left to right.

Rows not drawn. A row with `NA` or an infinite value of `x` has no place in the order and is
left out. A row with `NA` or an infinite value of `y` is left out too, and the line is broken
there: the row before it and the row after it are not joined to each other, so a gap in the
data shows as a gap in the line. A note counts the rows left out, and another the breaks. A
row left with no neighbour, the only row of its group or one between two rows without `y`,
draws nothing, and a note counts such rows; `points` over the same rows shows them.

Groups. The rows are parted by every variable that `group:`, `color:` and `linetype:` name,
together: a line for each combination that occurs. A row with `NA` in such a variable belongs
to a line of its own, in the grey of `NA` when the variable is shown by colour.

The types of line. The six are `solid`, `dashed`, `dotted`, `dash_dot`, `long_dash` and
`dash_dot_dot`, given to the categories in that order; with more than six categories the types
repeat from the seventh, and a note says so. The ink and the gaps are in widths of the line,
so a thicker line has longer dashes: 4 of ink and 3 of gap for `dashed`, 1 and 2 for `dotted`.
The ink begins where a line begins. The key of a legend is a piece of the line in its colour
and its type, and one legend serves a variable shown by colour and by type at once.

The corners of a line are drawn to a point, and cut flat where the point would reach more than
four widths beyond the corner; its ends are cut square. A figure written by `export` has the
same lines as PDF, as SVG and as PNG.

## See also
plot, points, export

## Examples
> let w = dataset(wages)
> let m = w.aggregate(wage = mean(wage), by: year)
warning[W0201]: 211 rows with NA in wage were skipped (line 2)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 2)
> m.plot(x: year, y: wage).lines()
plot · m · wage against year

  axis   title           scale    limits               breaks
  x      Calendar year   linear   2009.45 to 2021.55   2010, 2012, 2014, 2016, 2018, 2020
  y      wage            linear   21.9191 to 32.9442   22, 24, 26, 28, 30, 32

  layer   mark    data   marks   settings
      1   lines   m         12   width 1, 1 line
> let u = w.aggregate(wage = mean(wage), by: [year, union])
warning[W0201]: 211 rows with NA in wage were skipped (line 4)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 4)
> u.plot(x: year, y: wage, color: union).points().lines(linetype: union)
plot · u · wage against year

  axis   title           scale    limits               breaks
  x      Calendar year   linear   2009.45 to 2021.55   2010, 2012, 2014, 2016, 2018, 2020
  y      wage            linear   21.131 to 37.1199    25, 30, 35

  aesthetic   title          scale      values    shown as
  color       Union member   category   no, yes   orange, sky blue
  linetype    Union member   category   no, yes   solid, dashed

  layer   mark     data   marks   settings
      1   points   u         24   size 4.5, alpha 1
      2   lines    u         24   width 1, linetype by union, 2 lines
> let few = w.where(worker <= 6)
> few.plot(x: year, y: wage, group: worker).lines(width: 0.5)
plot · few · wage against year

  axis   title           scale    limits               breaks
  x      Calendar year   linear   2009.45 to 2021.55   2010, 2012, 2014, 2016, 2018, 2020
  y      Hourly wage     linear   11.4635 to 65.5065   20, 30, 40, 50, 60

  aesthetic   title               scale   values               shown as
  group       Worker identifier           each value a group

  layer   mark    data   marks   settings
      1   lines   few       30   width 0.5, 6 lines
> let gap = frame(t = [1, 2, 3, 4, 5], v = [1.0, 2.0, NA, 4.0, 3.0])
> gap.plot(x: t, y: v).lines()
plot · gap · v against t

  axis   title   scale    limits         breaks
  x      t       linear   0.8 to 5.2     1, 2, 3, 4, 5
  y      v       linear   0.85 to 4.15   1, 1.5, 2, 2.5, 3, 3.5, 4

  layer   mark    data   marks   settings
      1   lines   gap        4   width 1, 2 lines
note: 1 row with NA in t or v dropped (line 9)
note: 1 break in a line where v is NA (line 9)
