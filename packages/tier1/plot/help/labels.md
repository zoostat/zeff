# labels

## Summary
The title of a plot and the titles of its axes and legend

## Description
Gives a plot its words. Without `labels` a plot has no title, and each axis and each legend
is titled by the label of its variable, or by the name of the variable where it has no label.
`p.labels(...)` sets any of six: the `title:` of the figure and a `subtitle:` under it, the
titles of the two axes, `x:` and `y:`, the title of the legend of colour, `color:`, and a
`caption:` under the figure. It gives the plot back, so it is written in the chain with the
marks, before or after them.

A title says what the figure shows, and a subtitle what the reader should know to read it:
the unit, the sample, the year. A caption is for the source of the data or a note on a
method. Many journals set the title and the caption of a figure in the text of the article
and want none in the figure itself: then give the axes their titles and leave the rest out.

In R with ggplot2 this is `labs(title =, subtitle =, x =, y =, colour =, caption =)`. In
Stata it is `title()`, `subtitle()`, `xtitle()`, `ytitle()`, `legend(title())` and `note()`.

## Arguments
- p: The plot whose words are set.
- title: The title of the figure, set above everything else in the heavier face.
- subtitle: A line under the title, in the lighter face.
- x: The title of the `x` axis, in the place of the label of its variable.
- y: The title of the `y` axis.
- color: The title of the legend of colour.
- caption: A line under the figure, in the type of the labels.

## Value
The `Plot` with its words set. The plot given is not changed. What a later `labels` says of
one of the six takes the place of what an earlier one said of it, and leaves the other five
as they were.

## Details
Titles of axes. An axis is titled by the label of its variable, which `label` of the `data`
package sets and many files bring with them, so a frame whose variables are well labelled
needs no `labels` for its axes. An expression mapped to an axis is titled by the expression as
it is written, `log(wage)`, and a `case` by `case { … }`: these are the titles most worth
setting. The words of `x:` and `y:` come before both.

No title. An empty text takes a title away: `x: ""` leaves the `x` axis with its labels and
no title, and gives the room to the panel. `title: ""` takes back a title that an earlier
`labels` set.

The legend of colour. `color:` titles the legend of the variable mapped to `color:`. Where
the plot maps nothing to `color:` and fills its marks by `fill:`, it titles the legend of the
fills. A legend of sizes or of shapes by another variable keeps the label of its variable.

Long words. The title, the subtitle and the caption start at the left margin of the page
and are broken between words to its width; the title of an axis is broken to the length of
the axis, and that of a legend to a third of the page. A word longer than its line is not
broken. Each line takes its room from the panel, so a long title on a small figure leaves a
smaller panel, and a figure with no room left for it is refused when it is exported.

The type. The title is set at 10 points in the heavier face, the subtitle and the titles of
axes and legends at 9 points, and the caption at 8, the size of the labels of the axes. No
text of a figure is smaller than 8 points.

The printed plot shows the titles of the axes and of the legend in their rows, and the
title, the subtitle and the caption on its last lines.

## See also
plot, legend, scale, export

## Examples
> let d = frame(dose = [1, 2, 3, 4], change = [0.4, 1.1, 2.3, 2.9], arm = ["a", "b", "a", "b"])
> let p = d.plot(x: dose, y: change, color: arm).points()
> p.labels(title: "Change by dose", y: "Change in score", color: "Arm of the trial")
plot · d · change against dose

  axis   title             scale    limits           breaks
  x      dose              linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      Change in score   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  aesthetic   title              scale      values   shown as
  color       Arm of the trial   category   a, b     orange, sky blue

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1

title: Change by dose
> p.labels(x: "", caption: "Four patients, two in each arm.")
plot · d · change against dose

  axis   title    scale    limits           breaks
  x               linear   0.85 to 4.15     1, 1.5, 2, 2.5, 3, 3.5, 4
  y      change   linear   0.275 to 3.025   0.5, 1, 1.5, 2, 2.5, 3

  aesthetic   title   scale      values   shown as
  color       arm     category   a, b     orange, sky blue

  layer   mark     data   marks   settings
      1   points   d          4   size 4.5, alpha 1

caption: Four patients, two in each arm.
