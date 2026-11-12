# rug

## Summary
A tick at the margin of the panel for each row

## Description
Adds a rug to a plot: for each row, a short tick at the edge of the panel, at the row's place
on an axis. A rug shows where the values of one variable lie, each one of them, beside
whatever else the plot shows of it: under a scatter it shows the two distributions that the
cloud of points hides, and under a fitted curve it shows where the data are dense and where
the curve is held up by a few rows.

`sides:` says along which axis. `p.rug()` draws the ticks at the bottom of the panel, at the
`x` of each row. `sides: y` draws them at the left, at the `y` of each row, and `sides: both`
draws both.

A rug along `x` needs `x` alone, so `w.plot(x: wage).rug()` is a plot by itself: every value
of one variable, on a line.

`x:`, `y:` and `color:` of `rug` take the place, for this layer alone, of what the plot says
of them. With many rows the ticks lie on one another; `alpha:` lets the paper show through,
so that the darkness of the rug tells how many there are.

In R with ggplot2 this is `geom_rug()`, whose `sides = "b"`, `"l"` and `"bl"` are `x`, `y` and
`both`. In Stata it is `twoway rspike`, or the option `rug` of some plots.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to the position of each side of the rug.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot. What the layer names itself is read in this frame.
- x: For this layer alone, the variable or the expression whose values the ticks at the
  bottom stand at.
- y: The same for the ticks at the left.
- color: For this layer alone, as on `plot`: a bare name colours each tick by the value of a
  variable in its row, and a string sets one colour for every tick.
- sides: `x` for ticks at the bottom, at each value of `x`; `y` for ticks at the left, at each
  value of `y`; `both` for both.
- alpha: The opacity of a tick, from 0, which shows nothing, to 1, which is solid.

## Value
The `Plot` with the rug on top. The plot given is not changed, so a plot bound to a name can
be extended in more than one way.

## Details
The ticks. A tick is 4.5 points long, the diameter of a point, and 0.5 wide. It starts at the
axis and points into the panel. The axes leave a margin beyond the smallest and the largest
value, so a rug lies in that margin and seldom under the marks.

Rows not drawn. A rug along one axis draws every row that has a finite value there, whatever
the row holds on the other axis; a rug on both sides draws the rows that have both, which are
the rows `points` draws. A note counts the rows left out for `NA`, and another those with an
infinite value.

The axes. The values of a rug count for the limits of its axes as any mark's do. A rug along
`x` on a plot that maps nothing to `y` has an `x` axis and no other.

Colour. Without `color:` a rug takes the colour the plot gives, or the dark grey of the
theme's marks. A variable of categories gives each tick the colour of its category, and the
legend shows a piece of line for each. A number without value labels is shown on the
continuous scale, and a row without that number is left out and counted.

Ties. Rows with one value draw their ticks in one place. `alpha:` shows that there are
several; to spread them, add a little noise to the variable first.

## See also
points, hline, vline, plot

## Examples
> let d = frame(dose = [1, 2, 2, 4], change = [-1.2, 0.4, 1.1, 2.3])
> d.plot(x: dose, y: change).points().rug(sides: both)
plot · d · change against dose

  axis   title    scale    limits            breaks
  x      dose     linear   0.85 to 4.15      1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   -1.375 to 2.475   -1, 0, 1, 2

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1
      2   rug      d          4   sides both, alpha 1
> d.plot(x: change).rug(alpha: 0.5)
plot · d · change

  axis   title    scale    limits            breaks
  x      change   linear   -1.375 to 2.475   -1, 0, 1, 2

  layer   mark   data   marks   settings
      1   rug    d          4   sides x, alpha 0.5
