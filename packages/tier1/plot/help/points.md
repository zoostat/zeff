# points

## Summary
Points, one for each row

## Description
Adds a layer of points to a plot, one point for each row at its `x` and its `y`: a scatter
plot. `w.plot(x: educ, y: wage).points()` draws wage against education, and
`w.plot(x: educ, y: log(wage)).points()` draws its logarithm.

`alpha:` makes the points translucent, so that a crowded region shows as a darker one, and
`size:` sets their diameter. `jitter:` moves each point a little at random, which separates
points that fall on one another when a variable takes few distinct values, as years of education
do. The moves are drawn from the seeded stream, so `seed(n)` at the top of the script repeats
the plot exactly. `data:` gives the layer a frame of its own, for example a subset to be drawn
over the full data.

A point takes its colour from `color:` of the plot: the colour of its category, its place on
the ramp, or the one colour set for every mark. Without `color:` it has the colour of the theme.
In the same way it takes its size from `size:` of the plot, unless the layer has a `size:` of
its own, and its shape from `shape:` of the plot. Without them it is a circle 4.5 across.

A layer may name what it draws itself. `x:`, `y:`, `color:`, `size:` and `shape:` of `points`
are written as on `plot` and take the place, for this layer alone, of what the plot says of
them; what the layer does not name it takes from the plot. So
`d.plot(x: educ, y: wage).points().points(y: fitted, color: "firebrick")` draws the observed
wage and, over it, the fitted one, on the same axes.

In R with ggplot2 this is `geom_point()`, and with `jitter:` it is `geom_jitter()`; a layer's
own variables are its `aes()`, as in `geom_point(aes(y = fitted))`. In Stata it is
`scatter y x`, with its options `jitter()`, `mcolor()`, `msize()` and `msymbol()`, and two
variables on one plot are `twoway (scatter wage educ) (scatter fitted educ)`.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to `x` and one to `y`.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot: the variables the plot maps to `x` and `y`, placed as the plot
  places them, as numbers or as categories, those an expression it maps reads, as numbers, and
  the variables the plot maps to `color:`, `size:` and `shape:`, each of the same kind as in
  the frame of the plot. What the layer names itself
  is read in this frame.
- x: For this layer alone, the variable or the expression its points are placed by along `x`,
  in the place of the one the plot maps: a Bool, Int or Float variable of the layer's frame, or
  an expression over its variables.
- y: The same along `y`: `points(y: fitted)` draws another variable against the `x` of the
  plot.
- color: For this layer alone, as on `plot`: a bare name colours its points by a variable, and
  a string sets one colour for all of them, `"firebrick"` or `"#B22222"`.
- size: For this layer alone: a number is the diameter of every point, in points of 1/72 inch,
  above 0, and a bare name sizes the points by a variable, by area. It comes before `size:` of
  the plot. Without either a point is 4.5 across.
- shape: For this layer alone, as on `plot`: a bare name gives each category of a variable a
  shape, and a string sets one shape for every point, `"cross"` or `"square_open"`.
- alpha: The opacity of a point, from 0, invisible, to 1, solid.
- jitter: How far a point may move, as a share of the smallest gap between two values on its
  axis: 0 for none, and 0.4 for up to 0.4 of the gap either way.

## Value
The `Plot` with the layer of points on top. The plot given is not changed, so a plot bound to a
name can be extended in more than one way.

## Details
Rows not drawn. A row with `NA` in `x` or `y` is not drawn, and a note counts such rows; a row
with an infinite value is counted in a note of its own. A plot carries each note once: a later
layer whose note would read the same does not repeat it.

Colour. When the plot maps a variable to `color:`, each point has the colour of its row. A row
with `NA` in a variable shown as categories is drawn in the grey of the category `NA`. A row
with `NA` or an infinite value of a variable shown on a continuous scale is not drawn, and the
notes count it with the rows that lack `x` or `y`. A point has no inside of its own, so `fill:`
of the plot does not change it, and no row is left out for a missing `fill`.

Size. When the plot maps a variable to `size:`, the area of each point is in proportion to the
value of its row, and the largest value drawn is 18 across. A row with `NA` or an infinite
value of that variable is not drawn, and neither is a row with a value below 0. A row with a
value of 0 is kept as a point that cannot be seen. The notes count each kind. A layer with a
`size:` of its own draws every point at that size and does not read the variable: no row is
left out for it, and a frame given with `data:` need not hold it.

Shape. When the plot maps a variable to `shape:`, each point has the shape of its category, and
a row with `NA` is drawn as an asterisk. A string given to `shape:` of the plot is the shape of
every point. A plus, a cross and an asterisk are made of lines; the other shapes are solid, or
an outline when the name ends in `_open`.

A layer's own variables. The layers of a plot share one scale of each kind, whatever each
layer draws on it. An axis runs over the values of every layer, and its title is that of the
variable the plot maps, or, where the plot maps none, of the first a layer names. Two
variables mapped to `color:` by two layers share one legend and must be of one kind, both
categories or both numbers without value labels; categories that only a later layer has
follow those of the first. A layer that sets a colour, a size or a shape to one value does not
read the scale, and no row of it is left out for a missing value of the plot's variable. The
printed plot tells, for each layer, what it names itself.

Group and label. A point is drawn row by row and writes no text, so `group:` and `label:` of the
plot do not change it, and a frame given with `data:` need not hold them.

Expressions. When the plot maps an expression, as in `y: log(wage)`, a point is drawn at the
value the expression gives for its row. A layer on the frame of the plot draws the values
computed when the plot was started. A layer with `data:` computes the expression again on its
own frame, with the warnings of that computation, and a row where it gives `NA` or an infinite
value is not drawn.

Jitter. With `jitter:` j above 0, a point at x is drawn at x + (2u − 1)·j·r, where u is a
uniform draw on (0, 1) and r is the smallest gap between two different values of `x` among the
points drawn, or 1 when they are all equal. `y` is moved in the same way with its own draw and
its own gap. The draws are taken in the order of the rows, one for `x` and then one for `y`.
With j below 0.5 the clouds around two neighbouring values do not overlap. The limits of the
axes are taken after the points have moved, so every point lies within them. When no `seed` was
set, a note gives the seed chosen for the run.

## References
Chambers, Cleveland, Kleiner and Tukey (1983), Graphical Methods for Data Analysis, Wadsworth.
Cleveland (1993), Visualizing Data, Hobart Press.

## See also
plot, seed

## Examples
> seed(20261111)
> let w = dataset(wages)
> w.plot(x: educ, y: wage).points()
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 1
note: 211 rows with NA in educ or wage dropped (line 3)
> w.plot(x: educ, y: wage).points(alpha: 0.3, jitter: 0.4)
plot · w · wage against educ

  axis   title                scale    limits                breaks
  x      Years of schooling   linear   6.9615 to 21.0396     8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.91305 to 159.833   0, 50, 100, 150

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 0.3, jitter 0.4
note: 211 rows with NA in educ or wage dropped (line 4)
> let top = w.where(wage > 100)
note: 211 rows dropped where `wage > 100` was NA; `w.where(is_missing(wage))` lists them (line 5)
> w.plot(x: educ, y: wage).points(alpha: 0.3).points(data: top, size: 8)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 0.3
      2   points   top        26   size 8, alpha 1
note: 211 rows with NA in educ or wage dropped (line 6)
> let d = frame(dose = [1, 2, 3, 4, NA], response = [2.1, 3.9, 6.2, 7.8, 9.0])
> d.plot(x: dose, y: response).points()
plot · d · response against dose

  axis   title      scale    limits           breaks
  x      dose       linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      response   linear   1.815 to 8.085   2, 3, 4, 5, 6, 7, 8

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1
note: 1 row with NA in dose or response dropped (line 8)
> w.plot(x: educ, y: log(wage)).points(alpha: 0.3).points(data: top, size: 8)
plot · w · log(wage) against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      log(wage)            linear   1.07304 to 5.2147   2, 3, 4, 5

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 0.3
      2   points   top        26   size 8, alpha 1
note: 211 rows with NA in educ or log(wage) dropped (line 9)
> w.plot(x: educ, y: wage, color: union).points(alpha: 0.3).points(data: top, size: 8)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title          scale      values    shown as
  color       Union member   category   no, yes   orange, sky blue

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 0.3
      2   points   top        26   size 8, alpha 1
note: 211 rows with NA in educ or wage dropped (line 10)
> w.plot(x: educ, y: wage, color: female).points(data: top)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   10.6 to 19.4        12, 14, 16, 18
  y      Hourly wage          linear   97.874 to 154.986   100, 110, 120, 130, 140, 150

  aesthetic   title   scale      values         shown as
  color       Sex     category   male, female   orange, sky blue

  layer   mark     data   marks   settings
      1   points   top       26   size 4.5, alpha 1
> w.plot(x: educ, y: wage, size: hours, shape: union).points(alpha: 0.2).points(data: top, size: 8)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title          scale      values      shown as
  size        Hours worked   sqrt       0 to 69.2   by area, up to 18 across
  shape       Union member   category   no, yes     circle, triangle

  layer   mark     data    marks   settings
      1   points   w      18,167   size by hours, shape by union, alpha 0.2
      2   points   top        26   size 8, shape by union, alpha 1
note: 245 rows with NA in educ, wage or hours dropped (line 12)
> w.plot(x: educ, y: wage, size: hours).points(data: top)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   10.6 to 19.4        12, 14, 16, 18
  y      Hourly wage          linear   97.874 to 154.986   100, 110, 120, 130, 140, 150

  aesthetic   title          scale   values      shown as
  size        Hours worked   sqrt    0 to 59.5   by area, up to 18 across

  layer   mark     data   marks   settings
      1   points   top       26   size by hours, alpha 1
> let f = w.where(year == 2010).add(fitted = 3 + 2 * educ)
> f.plot(x: educ, y: wage).points(alpha: 0.4).points(y: fitted, color: "firebrick", size: 3)
plot · f · wage against educ

  axis   title                scale    limits            breaks
  x      Years of schooling   linear   7.4 to 20.6       8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -0.09 to 102.65   0, 20, 40, 60, 80, 100

  layer   mark     data   marks   settings
      1   points   f        531   size 4.5, alpha 0.4
      2   points   f        537   y fitted, color firebrick, size 3, alpha 1
note: 6 rows with NA in educ or wage dropped (line 15)
