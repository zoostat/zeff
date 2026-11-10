# points

## Summary
Points, one for each row

## Description
Adds a layer of points to a plot, one point for each row at its `x` and its `y`: a scatter
plot. `w.plot(x: educ, y: wage).points()` draws wage against education.

`alpha:` makes the points translucent, so that a crowded region shows as a darker one, and
`size:` sets their diameter. `jitter:` moves each point a little at random, which separates
points that fall on one another when a variable takes few distinct values, as years of education
do. The moves are drawn from the seeded stream, so `seed(n)` at the top of the script repeats
the plot exactly. `data:` gives the layer a frame of its own, for example a subset to be drawn
over the full data.

In R with ggplot2 this is `geom_point()`, and with `jitter:` it is `geom_jitter()`. In Stata it
is `scatter y x`, with its option `jitter()`.

## Arguments
- p: The plot to which the layer is added. It must map a variable to `x` and one to `y`.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold the
  variables the plot maps to `x` and `y`, as numbers.
- alpha: The opacity of a point, from 0, invisible, to 1, solid.
- size: The diameter of a point, in points of 1/72 inch. Without it a point is 4.5 across.
- jitter: How far a point may move, as a share of the smallest gap between two values on its
  axis: 0 for none, and 0.4 for up to 0.4 of the gap either way.

## Value
The `Plot` with the layer of points on top. The plot given is not changed, so a plot bound to a
name can be extended in more than one way.

## Details
Rows not drawn. A row with `NA` in `x` or `y` is not drawn, and a note counts such rows; a row
with an infinite value is counted in a note of its own. A plot carries each note once: a later
layer whose note would read the same does not repeat it.

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

  axis   title                scale    limits               breaks
  x      Years of schooling   linear   6.9619 to 21.0316    8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.9144 to 159.834   0, 50, 100, 150

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
