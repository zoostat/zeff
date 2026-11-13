# line

## Summary
A line plot of a frame: `y` against `x`, the rows joined in the order of `x`

## Description
Draws `y` against `x` as a line, in one call: the rows are put in the order of `x` and each is
joined to the next. `m.line(year, wage)` shows the wage from year to year. It is the plot to
begin with for a series in time, for a dose and its response, for anything measured along an
ordered scale.

It is a shorthand of the grammar: `m.line(year, wage, color: union)` is
`m.plot(x: year, y: wage, color: union).lines()`, the same `Plot` value. So everything a plot
takes may follow it in the chain: `points()` for a mark at each row, `scale_y(log)`, `labels`,
`theme`, `export`.

`color:` draws a line for each category of a variable, each in its colour, with a legend.
`by:` gives each value of a variable a panel of its own, as `facet(by:)` does.

In R this is `plot(x, y, type = "l")` on rows in order, and with ggplot2 `geom_line()`. In
Stata it is `line y x, sort`.

## Arguments
- data: The frame whose rows are drawn.
- x: The variable, or the expression, along the foot of the plot, by which the rows are
  ordered: a number, a date, or categories.
- y: The variable, or the expression, up the side.
- color: A variable shown as categories; a line for each of them, each in its colour. A line
  has one colour, so a number without value labels is refused.
- by: A variable, or several in brackets, whose values part the plot into panels with the
  same axes.

## Value
A `Plot`, the one its grammar form gives. It is drawn where it is printed and may be bound,
extended and exported.

## Details
One row for each `x`. A line reads well where each line has one row at each value of `x`.
With many rows at one `x`, as in the wages of many workers in one year, the line goes up and
down at that `x`; compute one value for each first, as in
`w.aggregate(wage = mean(wage), by: year).line(year, wage)`.

Rows not drawn. A row with `NA` or an infinite value of `x` is left out. A row with `NA` or
an infinite value of `y` is left out too and the line is broken there, so a gap in the data
shows as a gap in the line. Notes count the rows left out and the breaks.

A line for each unit. `color:` parts the rows by a variable shown in colours, which suits a
few categories. For a line for each of many units, a worker or a country, write the grammar
form with `group:`, as in `w.plot(x: year, y: wage, group: worker).lines(alpha: 0.2)`.

What the shorthand does not take. The type and the width of the lines, a line in steps, a
second layer with a frame of its own: these are written in the grammar form, or added to the
shorthand in the chain. A refusal of the shorthand is the refusal of `plot`, of `lines` or of
`facet`, and points at the argument of `line` that caused it.

## See also
plot, lines, step, facet, scatter, aggregate

## Examples
> let m = frame(year = [2019, 2020, 2021, 2019, 2020, 2021], rate = [4.1, 6.3, 5.0, 3.2, 5.9, 4.4], region = ["north", "north", "north", "south", "south", "south"])
> m.line(year, rate, color: region)
plot · m · rate against year

  axis   title   scale    limits             breaks
  x      year    linear   2018.9 to 2021.1   2019, 2019.5, 2020, 2020.5, 2021
  y      rate    linear   3.045 to 6.455     3.5, 4, 4.5, 5, 5.5, 6

  aesthetic   title    scale      values         shown as
  color       region   category   north, south   orange, sky blue

  layer   mark    data   marks   settings
      1   lines   m          6   width 1, 2 lines
> m.line(year, rate, by: region)
plot · m · rate against year

  axis   title   scale    limits             breaks
  x      year    linear   2018.9 to 2021.1   2019, 2019.5, 2020, 2020.5, 2021
  y      rate    linear   3.045 to 6.455     3.5, 4, 4.5, 5, 5.5, 6

  layer   mark    data   marks   settings
      1   lines   m          6   width 1, 2 lines

panels: 2 by region: north, south
