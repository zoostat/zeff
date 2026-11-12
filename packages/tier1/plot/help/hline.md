# hline

## Summary
Level reference lines, at a value or at each value of a variable

## Description
Adds a level reference line to a plot: a line from the left edge of the panel to the right,
at a place on the `y` axis. A reference line gives the eye something to read the marks
against: zero for a difference, a target, the mean of a group, a threshold.

`at:` says where. A number draws one line, as in `p.hline(at: 0)`. The name of a
variable draws a line at each of its values, one for each row of the frame, so a small frame of
places, given as `data:`, draws several lines at once.

A reference line belongs to the plot's axes and to nothing else of the plot. It does not take
the plot's colour, and a frame given as `data:` needs none of the plot's variables. `color:`
gives the lines a colour of their own, `style:` a type of line, dashed unless another is
asked for, and `width:` a width.

The `y` axis is made wide enough to hold every reference line, so a line is never out of
sight. `vline` is the same along the other axis.

In R with ggplot2 this is `geom_hline(yintercept = )`. In Stata it is the option `yline()` of a
`twoway` graph.

## Arguments
- p: The plot to which the layer is added. With a number for `at:`, the plot or a layer
  before this one must map something to `y`, so that there is an axis to place the line on.
- data: A frame for this layer alone, in place of the frame of the plot: the frame whose
  variable `at:` names. It needs none of the variables the plot maps.
- at: Where the lines stand on the `y` axis. A number draws one line at that place. A
  bare name draws a line at each value of that variable, which must be numeric.
- color: A string sets one colour for every line of the layer. A bare name colours each line
  by the value of a variable in its row, which needs `at:` to name a variable too.
- style: The type of the lines: `solid`, `dashed`, `dotted`, `dash_dot`, `long_dash` or
  `dash_dot_dot`, the six types of `lines`, written as a word without quotation marks.
- width: The width of a line, in points of 1/72 inch, above 0.

## Value
The `Plot` with the layer of reference lines on top. The plot given is not changed, so a plot
bound to a name can be extended in more than one way.

## Details
A number or a variable. A bare name given to `at:` is always read as a variable of the frame.
A number computed earlier in the script and bound to a name is given as an expression that
holds the name, as in `at: centre + 0`.

Rows not drawn. With a variable, a row whose value is `NA` or infinite draws no line, and a
note counts such rows. Two rows with one value draw two lines in one place.

The axis. The values of `at:` count for the limits of the `y` axis as the marks do: a
line at 0 on a plot whose marks start at 5 brings 0 onto the axis. Where the plot maps nothing
to `y` and `at:` names a variable, the axis is that variable's, with its name for a title.

Colour. Without `color:` a reference line is drawn in the dark grey of the theme's marks.
With a variable of categories each line has the colour of its category, and the legend shows
a piece of line for each; with a number without value labels, a place on the continuous
scale. The variable must be of the kind the plot's colour scale already shows, since the
layers share one scale of colour.

Order. A layer is drawn over the ones before it, so `p.hline(at: 0).points()` puts the line
under the points and `p.points().hline(at: 0)` over them.

## See also
vline, lines, plot

## Examples
> let d = frame(dose = [1, 2, 3, 4], change = [-1.2, 0.4, 1.1, 2.3])
> d.plot(x: dose, y: change).hline(at: 0).points()
plot · d · change against dose

  axis   title    scale    limits            breaks
  x      dose     linear   0.85 to 4.15      1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   -1.375 to 2.475   -1, 0, 1, 2

  layer   mark     data   marks   settings
      1   hline    d          1   at 0, width 0.5, style dashed
      2   points   d          4   size 4.5, alpha 1
> let limits = frame(level = [-1, 1], kind = ["lower", "upper"])
> d.plot(x: dose, y: change).points().hline(data: limits, at: level, color: kind, style: dotted)
plot · d · change against dose

  axis   title    scale    limits            breaks
  x      dose     linear   0.85 to 4.15      1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   -1.375 to 2.475   -1, 0, 1, 2

  aesthetic   title   scale      values         shown as
  color       kind    category   lower, upper   orange, sky blue

  layer   mark     data     marks   settings
      1   points   d            4   size 4.5, alpha 1
      2   hline    limits       2   at level, color by kind, width 0.5, style dotted
