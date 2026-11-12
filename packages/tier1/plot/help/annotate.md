# annotate

## Summary
A text written at a place of the data

## Description
Writes a few words at a place on the two axes: a name beside a point that matters, the date
of an event on a series, the word for a region of the panel. `p.annotate(text: "...", x:, y:)`
gives the words and the place, in the units of the data, so that the text stays with the data
whatever the size of the figure. It gives the plot back, so it is written in the chain with
the marks, and it is drawn over the layers written before it.

`annotate` is for one text written in the script. To write a text for each row of a frame,
the name of each country at its point, use `text`.

In R with ggplot2 this is `annotate("text", x =, y =, label =)`. In Stata it is the option
`text(y x "...")`.

## Arguments
- p: The plot the text is written on. Something must be mapped to `x` and to `y`, so that
  there are axes to place it by.
- text: The words, a string.
- x: The place along the `x` axis: a number on an axis of numbers, the name of a category
  on an axis of categories, a date on an axis of dates.
- y: The place along the `y` axis, in the same forms.
- size: The size of the type, in points of 1/72 inch, above 0. Left out, it is 8 points, the
  size of the labels of the axes.

## Value
The `Plot` with the text as a layer on top. The plot given is not changed.

## Details
The place. The text is centred on its place, along both axes. A place is of the kind its
axis holds: a number where the axis is of numbers, as in `x: 2015`; the name of a category
that the axis has, as in `x: "south"`; a date, written with `date` or `datetime`, on an axis
of dates. A place of another kind is refused, and so is a category the axis does not have.

A layer of one row. An annotation is a layer of text of one row, in a frame of its own called
`annotation`, and the printed plot lists it with its words and its place. As any mark does,
it counts for the limits of the axes: a text placed beyond the data widens the axis to hold
it. On a log scale its place must be above 0.

At the edge. A text is cut off at the edge of the panel, as the marks are. Words near an edge
are best placed a little further in, or the limits of the axis widened with `scale_x` or
`scale_y`.

The type. The text is set in the face of the labels, in the dark grey of the theme. It is not
made larger by the base size of a theme: `size:` sets it.

## See also
text, labels, hline, vline, plot

## Examples
> let d = frame(dose = [1, 2, 3, 4], change = [0.4, 1.1, 2.3, 2.9])
> d.plot(x: dose, y: change).lines().annotate(text: "steepest here", x: 2.5, y: 1.2)
plot · d · change against dose

  axis   title    scale    limits           breaks
  x      dose     linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  layer   mark    data         marks   settings
      1   lines   d                4   width 1, 1 line
      2   text    annotation       1   "steepest here" at 2.5, 1.2, size 8
> let s = frame(shop = ["north", "south", "east"], units = [12, 18, 15])
> s.plot(x: shop, y: units).bars().annotate(text: "best month", x: "south", y: 19, size: 10)
plot · s · units against shop

  axis   title   scale      limits           breaks
  x      shop    category   0.4 to 3.6       east, north, south
  y      units   linear     -0.95 to 19.95   0, 5, 10, 15

  layer   mark   data         marks   settings
      1   bars   s                3   width 0.9, alpha 1
      2   text   annotation       1   "best month" at south, 19, size 10
