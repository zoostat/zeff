# plot

## Summary
Start a layered plot

## Description
Starts a plot of a frame: `w.plot(x: educ, y: wage)` names the frame and the variables placed
along the two axes. In place of a variable an axis takes an expression of the variables, as in
`y: log(wage)`. A plot is built in layers. `plot` draws nothing by itself; each mark that
follows, such as `.points()`, adds a layer on top of those before it, and all the layers share
the frame, the mapped variables and the scales. A layer may name a variable of its own in the
place of one the plot maps, as in `.points(y: fitted)`, and a frame of its own with `data:`;
the scales stay shared.

`color:` shows a third variable by the colour of the marks. `color: female` gives each category
a colour of its own, and `color: hours` shades the marks along a ramp from the smallest value to
the largest. A string sets one colour for every mark instead, as in `color: "steelblue"`.
`fill:` is the colour of the inside of a mark that has one.

`size:` shows a variable that is an amount by the area of the marks, as in `size: hours`, and a
number sets one size for every mark, as in `size: 6`. `shape:` gives each category of a
variable a shape of its own, as in `shape: union`, and a string sets one shape for every mark,
as in `shape: "square"`. `group:` names the variable whose values gather the rows into groups,
and `label:` the variable whose values are written as text. Both are kept for the marks that
read them.

A plot is a value. It can be bound with `let`, extended later and printed. Printed, it reads as
the text of its scene: a row for each axis, with its title, its limits and its breaks, a row
for each colour, size and shape, with its values and what they are shown as, and a row for each
layer, with the number of marks it draws.

Each variable mapped to an axis is placed on a linear scale, so it must be a Float, an Int, or
a Bool, which is read as 0 and 1, and an expression must give one of these for each row. The
title of an axis and of every other scale is the label of its variable, or the name of the
variable when it has no label; the title of an expression is the expression as it is written.

In R with ggplot2 this is `ggplot(w, aes(x = educ, y = wage, colour = female))`, where
`aes(y = log(wage))` maps an expression in the same way, and in Stata it is the start of a
`twoway` command, for which the variable is generated first and each group is drawn by a
`scatter` of its own. The defaults of the scales follow Observable Plot. The default colours
are not those of ggplot2: the categories take the colours of `palette.colors(palette =
"Okabe-Ito")` in R, with black moved to the last place, and a continuous scale takes those of
`scale_colour_viridis_c()`. A size follows `scale_size_area()` of ggplot2 and the weighted
markers of Stata, as in `scatter wage educ [aweight = hours]`, in which a value of 0 has no
size. It does not follow the default `scale_size()` of ggplot2, which gives the smallest value
a size above 0. The first three shapes, a circle, a triangle and a square, are the first three
of ggplot2.

## Arguments
- data: The frame whose rows are drawn. A layer may name a frame of its own with `data:`.
- x: The variable placed along the horizontal axis, a bare name, or an expression of the
  variables such as `log(wage)` or `wage / hours`.
- y: The variable placed along the vertical axis, a bare name, or an expression of the
  variables.
- color: The colour of the marks. A variable of the frame, written as a bare name, is shown on
  a colour scale. A string sets one colour for every mark: a name in small letters such as
  `"steelblue"`, or `#` and six hexadecimal digits such as `"#4682B4"`. Without it the marks
  have the colour of the theme.
- fill: The colour of the inside of the marks that have one, given as `color:` is given. A
  point has no inside and takes its colour from `color:`.
- size: The size of the marks. A variable of the frame, written as a bare name, is shown by the
  area of each mark; it must be an Int or a Float without value labels. A number sets the
  diameter of every mark in points of 1/72 inch and must be above 0. Without it a point is 4.5
  across.
- shape: The shape of the marks. A variable of the frame, written as a bare name, gives each of
  its categories a shape. A string sets one shape for every mark: `"circle"`, `"triangle"`,
  `"square"`, `"diamond"`, `"plus"`, `"cross"`, `"triangle_down"`, `"star"` or `"asterisk"`,
  and each of those with an inside also as an outline, as in `"circle_open"`. Without it a
  mark is a circle.
- group: The variable whose values gather the rows into groups, for the marks that draw one
  thing through several rows. A point is drawn row by row and does not read it.
- label: The variable whose values are the text of the marks that write one. A point writes
  none.

## Value
A `Plot`, to which marks are added as methods, each call giving a new plot with one more layer.
Printed, it shows the frame and the mapped variables, then the axes with their scales, then the
colours, the sizes, the shapes, the group and the label, then the layers in the order in which
they are drawn.

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

Colour by categories. A Category, a Str and a Bool variable are shown as categories, and so is
an Int or a Float that carries value labels. Each category has a colour of its own. The
categories of a Category come in the order of its levels. Those of another variable come in the
order of its values, `false` before `true`, numbers by size and text by the code points of its
characters, and each is named by its value label when it has one. The colours are the eight of
Okabe and Ito, chosen so that people with the common forms of colour blindness can tell them
apart, in this order: orange, sky blue, bluish green, yellow, blue, vermilion, reddish purple
and black. With more than eight categories the colours repeat from the ninth, and a note says
so. A row with `NA` is drawn: `NA` is a category of its own, which comes last and is grey.

The scale shows the categories of the marks drawn, so a level with no row among them has no
colour and no place in the legend. Before any layer is added it shows the categories the frame
holds.

Colour on a continuous scale. An Int or a Float without value labels is shown on the ramp
viridis, which runs from dark purple through blue and green to yellow. Its lightness grows
evenly, so it keeps its order when printed in grey and for colour-blind readers. The scale
runs from the smallest value drawn to the largest, with no margin, and is cut into 256 equal
parts, each with its colour; a mark takes the colour of the part in which its value lies. A
row with `NA` or an infinite value of the variable has no place on the ramp: a layer that reads
the colour does not draw it and counts it in its note. When all the values are one value the
limits follow the rule of the axes.

One colour for every mark. A string sets the colour of every mark and needs no scale and no
legend. The names are the 148 of CSS, all in small letters, with both `"grey"` and `"gray"`.
After `#` come the red, the green and the blue as two hexadecimal digits each, and `"#48B"` is
short for `"#4488BB"`. A bare name is always read as a variable of the frame, so the string is
written where it is used.

What a colour cannot hold. An expression is not mapped to a colour, nor to `fill:`, `size:`,
`shape:`, `group:` or `label:`, which take a variable by its name: `color: wage > 20` is an
error (Z0401) that says so. Compute the variable with `add` first, as in
`w.add(high = wage > 20).plot(x: educ, y: wage, color: high)`. A number worked out from numbers
alone, `size: 2 + 1`, is a number. A Date and a DateTime have no colour scale. When a layer
has a frame of its own, the variable must be of the same kind there, categories or a number
without value labels, and categories that only that frame holds come after those of the frame
of the plot. The same holds for a size and for a shape.

Size. A variable mapped to `size:` is an amount: an Int or a Float without value labels. The
area of a mark is in proportion to its value, so a value four times another is drawn twice as
wide. The scale starts at 0, where a mark has no size, and the largest value drawn is 18 points
across, four times the size of a point without `size:`. A value v is drawn 18 × √(v / m)
across, where m is the largest value drawn. The legend shows the breaks above 0. A row with
`NA` or an infinite value of the variable is not drawn, and neither is a row with a value below
0, for which there is no area. A row with a value of 0 is kept as a mark that cannot be seen.
Each layer counts such rows in its notes.

One size for every mark. A number given to `size:` is the diameter of every mark and needs no
scale and no legend. A number bound to a name is not read here, because a bare name is always a
variable of the frame, so the number is written where it is used. A `size:` given to a layer
comes before what the plot says, for that layer alone.

Shape. A variable mapped to `shape:` must be one that is shown as categories: a Category, a
Str, a Bool, or an Int or a Float with value labels. The categories come in the order
described under colour and take in turn a circle, a triangle, a square, a diamond, a plus, a
cross, a triangle on its point and a five-pointed star: eight shapes, one for each colour of
the palette. The two hardest to tell from an earlier shape come last, so that only a seventh
and an eighth category meet them. With more than eight categories the shapes repeat from the
ninth, and a note says so. `NA` is a category of its own, which comes last and is drawn as an
asterisk.

One shape for every mark. A string names the shape of every mark. A circle, a triangle, a
square, a diamond, a triangle on its point (`"triangle_down"`) and a star are drawn solid, and
with `_open` after the name, as in `"square_open"` or `"star_open"`, as an outline. A plus, a
cross and an asterisk are made of lines.

One legend for a variable. A variable mapped to more than one of `color:`, `size:` and
`shape:`, as in `color: union, shape: union`, has one legend, whose keys show all of them.

Group and label. `group:` and `label:` take a variable of any type and need no scale. A printed
plot lists them, and says so when no layer draws them.

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
axes, the legends and the layout. It has one legend for each variable that a layer shows by a
colour, a size or a shape. It holds no size in pixels and no fonts, which belong to the program
that draws it, so the same data give the same scene on every computer.

A figure. `p.export("output/wage.pdf", size: one_column)` writes the plot as a figure, in the
format its extension names: `.pdf` or `.svg`, drawn with lines, or `.png`, drawn in dots. The
scene is laid out on a page of that size (`one_column`, `two_column`, `slide`, or a width and a
height in millimetres), under one theme: a white page, the major gridlines in light grey, the
two axes as thin lines with their ticks outside the panel, labels of 8 points and titles of 9
in IBM Plex Sans, and the legends at the right. A title too long for its side is broken between
words, and labels of breaks that would touch are thinned, the ticks and gridlines staying. A
mark at the edge of the panel is cut by it. Shapes of one size have one area, so that no
category looks heavier than another. The notes on the rows not drawn are printed with the plot
and stay off the figure unless `export` is given `notes: true`. The page of `export` describes
the file.

Either `x:` or `y:` may be left out. A mark then says which of the two it needs.

## References
Wilkinson (2005), The Grammar of Graphics, second edition, Springer.
Wickham (2010), A layered grammar of graphics, Journal of Computational and Graphical
Statistics 19, 3–28.
Bostock, Ogievetsky and Heer (2011), D3: data-driven documents, IEEE Transactions on
Visualization and Computer Graphics 17, 2301–2309.
Okabe and Ito (2008), Color Universal Design: how to make figures and presentations that are
friendly to colorblind people, J*Fly, University of Tokyo.
Wong (2011), Points of view: color blindness, Nature Methods 8, 441.
Smith and van der Walt (2015), A better default colormap for Matplotlib, SciPy 2015.

## See also
points, export, describe, label

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
> w.plot(x: educ, y: wage, color: female).points(alpha: 0.3)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title   scale      values         shown as
  color       Sex     category   male, female   orange, sky blue

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 0.3
note: 211 rows with NA in educ or wage dropped (line 9)
> w.plot(x: educ, y: wage, color: hours).points()
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title          scale    values      shown as
  color       Hours worked   linear   7 to 69.2   viridis, dark purple to yellow

  layer   mark     data    marks   settings
      1   points   w      18,167   size 4.5, alpha 1
note: 245 rows with NA in educ, wage or hours dropped (line 10)
> w.plot(x: educ, y: wage, color: "steelblue").points()
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title   scale      values       shown as
  color               constant   every mark   steelblue

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, alpha 1
note: 211 rows with NA in educ or wage dropped (line 11)
> w.plot(x: educ, y: wage, size: hours).points(alpha: 0.3)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title          scale   values      shown as
  size        Hours worked   sqrt    0 to 69.2   by area, up to 18 across

  layer   mark     data    marks   settings
      1   points   w      18,167   size by hours, alpha 0.3
note: 245 rows with NA in educ, wage or hours dropped (line 12)
> w.plot(x: educ, y: wage, color: union, shape: union).points()
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title          scale      values    shown as
  color       Union member   category   no, yes   orange, sky blue
  shape       Union member   category   no, yes   circle, triangle

  layer   mark     data    marks   settings
      1   points   w      18,201   size 4.5, shape by union, alpha 1
note: 211 rows with NA in educ or wage dropped (line 13)
> w.plot(x: educ, y: wage, size: 3, shape: "cross").points()
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title   scale      values       shown as
  size                constant   every mark   3
  shape               constant   every mark   cross

  layer   mark     data    marks   settings
      1   points   w      18,201   size 3, shape cross, alpha 1
note: 211 rows with NA in educ or wage dropped (line 14)
> w.plot(x: educ, y: wage, group: worker, label: firm)
plot · w · wage against educ

  axis   title                scale    limits              breaks
  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20
  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150

  aesthetic   title               scale   values               shown as
  group       Worker identifier           each value a group
  label       Firm identifier             the text of a mark

no layer yet: a mark such as `.points()` adds one

