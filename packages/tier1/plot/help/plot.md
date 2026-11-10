# plot

## Summary
Start a layered plot

## Description
Starts a plot of a frame: `w.plot(x: educ, y: wage)` names the frame and the variables placed
along the two axes. A plot is built in layers. `plot` draws nothing by itself; each mark that
follows, such as `.points()`, adds a layer on top of those before it, and all the layers share
the frame, the mapped variables and the scales.

A plot is a value. It can be bound with `let`, extended later and printed. Printed, it reads as
the text of its scene: a row for each axis, with its title, its limits and its breaks, and a row
for each layer, with the number of marks it draws.

Each mapped variable is placed on a linear scale, so it must be a Float, an Int, or a Bool,
which is read as 0 and 1. The title of an axis is the label of its variable, or the name of the
variable when it has no label.

In R with ggplot2 this is `ggplot(w, aes(x = educ, y = wage))`, and in Stata it is the start of
a `twoway` command. The defaults of the scales follow Observable Plot.

## Arguments
- data: The frame whose rows are drawn. A layer may name a frame of its own with `data:`.
- x: The variable placed along the horizontal axis, a bare name.
- y: The variable placed along the vertical axis, a bare name.

## Value
A `Plot`, to which marks are added as methods, each call giving a new plot with one more layer.
Printed, it shows the frame and the mapped variables, then the axes with their scales, then the
layers in the order in which they are drawn.

## Details
Scales. The limits of an axis lie 5% of the range beyond the smallest and the largest value
drawn, so that no mark sits on the edge of the panel. Before any layer is added they are taken
from every value of the variable in the frame. When all the values are one value v, the limits
are v minus and plus the larger of 0.5 and 5% of |v|; with no value at all they are 0 and 1.

Breaks. The breaks are the multiples of a step that lie within the limits. The step is 1, 2 or
5 times a power of ten, chosen so that the limits span about five steps; it is the rule of
Observable Plot and of D3. The labels of the breaks are numbers as tables print them, to six
significant digits.

Missing values. A row with `NA` in a mapped variable cannot be placed and is not drawn, and
neither is a row with an infinite value. Each layer counts the rows it leaves out in a note.

The scene. A plot is kept as a scene: the marks in the units of the data, with the scales, the
axes and the layout. It holds no size in pixels and no fonts, which belong to the program that
draws it, so the same data give the same scene on every computer.

Either `x:` or `y:` may be left out. A mark then says which of the two it needs.

## References
Wilkinson (2005), The Grammar of Graphics, second edition, Springer.
Wickham (2010), A layered grammar of graphics, Journal of Computational and Graphical
Statistics 19, 3–28.
Bostock, Ogievetsky and Heer (2011), D3: data-driven documents, IEEE Transactions on
Visualization and Computer Graphics 17, 2301–2309.

## See also
points, describe, label

## Examples
> let w = dataset(wages)
> w.plot(x: educ, y: wage)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

no layer yet: a mark such as `.points()` adds one
> let p = w.plot(x: educ, y: wage).points(alpha: 0.3)
note: 211 rows with NA in educ or wage dropped (line 3)
> p
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 0.3
> w.plot(x: hours)
plot · w · hours

  axis   title          scale    limits          breaks
  x      Hours worked   linear   3.89 to 72.31   10, 20, 30, 40, 50, 60, 70

no layer yet: a mark such as `.points()` adds one
