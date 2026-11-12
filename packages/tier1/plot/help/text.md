# text

## Summary
The label of each row written at its place

## Description
Adds a layer of text to a plot: for each row, the value of the variable `label:` names,
written at the row's `x` and `y`. Text names the marks of a plot: the country of each point of
a scatter, the value on top of each bar, the industry at the end of each line.

`p.points().text(label: name)` writes each name on its point, which hides both. `nudge:`
moves the labels off their places: `nudge: [0, 6]` writes each label 6 points higher, clear
of a point drawn there. `size:` is the size of the type and `color:` its colour, one colour
for every label or the colour of a variable in each row.

Every row has a label, so text suits a few rows, the ones worth naming. A layer may have a
frame of its own for that: `p.points().text(data: top, label: name)` names the rows of `top`
alone.

In R with ggplot2 this is `geom_text()`, whose `nudge_x` and `nudge_y` move a label in the
units of the data and not in points. In Stata it is the option `mlabel()` of `scatter`.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to `x` and one to `y`, and a variable to `label`.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot. What the layer names itself is read in this frame.
- x: For this layer alone, the variable or the expression the labels are placed by along `x`.
- y: The same along `y`.
- label: The variable whose values are written, for this layer alone, in place of `label:` of
  the plot. Any type will do: a number is written as it prints, and a value with a value
  label as its label.
- color: For this layer alone, as on `plot`: a bare name colours each label by the value of
  a variable in its row, and a string sets one colour for every label.
- size: The size of the type, in points of 1/72 inch, above 0.
- nudge: A list of two numbers: how far every label is moved from its place, to the right
  and up, in points of 1/72 inch. A number below zero moves it left or down.

## Value
The `Plot` with the layer of text on top. The plot given is not changed, so a plot bound to a
name can be extended in more than one way.

## Details
The place. A label is centred on its place, in both directions, and then moved by `nudge:`.
The move is in points on the page and not in the units of the data, so it is the same on
every scale: 6 points clear a point of the usual size whatever the axes hold. The axes take
their limits from the places, not from the labels, so a long label near an edge of the panel
is cut off there; nudge it inwards, or widen the axis.

Rows not drawn. A row with `NA` or an infinite value in `x` or `y` writes nothing, and
neither does a row without a label, or without the number its colour shows on a continuous
scale; notes count them. A label that is empty text is written, and shows nothing.

The type. The text is set in the sans-serif face of the theme, 8 points high unless `size:`
says otherwise, as the labels of the axes are. A label is written on one line, however long.

Overlap. Labels are written where their rows are, and labels of rows that lie close are
written on one another. Name fewer rows, or move some by giving them a layer of their own with
another `nudge:`.

## See also
points, plot, hline, vline

## Examples
> let c = frame(income = [12, 30, 45], life = [61, 72, 80], name = ["Aland", "Borduria", "Carpania"])
> c.plot(x: income, y: life).points().text(label: name, nudge: [0, 7])
plot · c · life against income

  axis   title    scale    limits           breaks
  x      income   linear   10.35 to 46.65   20, 30, 40
  y      life     linear   60.05 to 80.95   65, 70, 75, 80

  layer   mark     data   marks   settings
      1   points   c          3   size 4.5, alpha 1
      2   text     c          3   label name, size 8, nudge 0, 7
