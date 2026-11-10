# plot

## Summary
Start a layered plot

## Description
Starts a plot of a frame: `w.plot(x: educ, y: wage)` names the frame and the variables placed
along the two axes. In place of a variable an axis takes an expression of the variables, as in
`y: log(wage)`. A plot is built in layers. `plot` draws nothing by itself; each mark that
follows, such as `.points()`, adds a layer on top of those before it, and all the layers share
the frame, the mapped variables and the scales.

A plot is a value. It can be bound with `let`, extended later and printed. Printed, it reads as
the text of its scene: a row for each axis, with its title, its limits and its breaks, and a row
for each layer, with the number of marks it draws.

Each mapped variable is placed on a linear scale, so it must be a Float, an Int, or a Bool,
which is read as 0 and 1, and an expression must give one of these for each row. The title of
an axis is the label of its variable, or the name of the variable when it has no label; the
title of an expression is the expression as it is written.

In R with ggplot2 this is `ggplot(w, aes(x = educ, y = wage))`, where `aes(y = log(wage))`
maps an expression in the same way, and in Stata it is the start of a `twoway` command, for
which the variable is generated first. The defaults of the scales follow Observable Plot.

## Arguments
- data: The frame whose rows are drawn. A layer may name a frame of its own with `data:`.
- x: The variable placed along the horizontal axis, a bare name, or an expression of the
  variables such as `log(wage)` or `wage / hours`.
- y: The variable placed along the vertical axis, a bare name, or an expression of the
  variables.

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

Expressions. An expression mapped to an axis is computed for each row as `add` computes it,
from the variables of the frame, numbers, the operators and the functions of `core`:
`log(wage)`, `wage / hours`, `exper^2`. The axis shows its values and the frame is not changed.
A row where it gives `NA` or an infinite value is not drawn, and the warnings of the
computation, such as that of the logarithm of zero, count such rows. A name that is not a
variable of the frame is read as a binding of the script, with the value it holds where `plot`
is called; a later change of the binding does not change the plot. A number alone, as in
`y: 0`, is that number in every row.

What an expression cannot hold. A summary of a variable, such as `mean(wage)`, a function of
another package, a function defined in the script, `case` and `if` are not computed inside a
mapping, and neither are `clamp`, `logit`, `inv_logit` and `fill_missing`. Compute such a
value with `add` first and map the new variable.

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
> w.plot(x: educ, y: log(wage)).points()
plot · w · log(wage) against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      log(wage)            linear   1.07304 to 5.2147   2, 3, 4, 5

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 1
note: 211 rows with NA in educ or log(wage) dropped (line 6)
> let weekly = 40
> w.plot(x: exper^2, y: wage * weekly)
plot · w · wage * weekly against exper^2

  axis   title           scale    limits               breaks
  x      exper^2         linear   -64.8 to 1360.8      0, 200, 400, 600, 800, 1000, 1200
  y      wage * weekly   linear   -156.52 to 6393.32   0, 1000, 2000, 3000, 4000, 5000, 6000

no layer yet: a mark such as `.points()` adds one
