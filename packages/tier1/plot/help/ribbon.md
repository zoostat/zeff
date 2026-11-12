# ribbon

## Summary
The band between two variables, filled

## Description
Adds a layer of bands to a plot: at each row the band reaches from `low:` to `high:`, and
the rows are put in the order of `x` and joined as `lines` joins them. A band shows a range
around a line: a confidence interval around a fitted line, the spread between a lowest and a
highest value, the quartiles around a median.

`p.ribbon(low: lower, high: upper)` draws the band under whatever is added after it, so
`p.ribbon(low: lower, high: upper).lines()` shows a line with its band. The band is filled
lightly, with an `alpha:` of 0.3, so that the gridlines and the marks under it stay visible.

One band is drawn for each group, as one line is: `group:` names the variable whose values
part the rows into bands, and a variable shown by `fill:` parts them too, each band in the
colour of its category.

A band has no outline. `p.lines(y: lower).lines(y: upper)` draws its two edges.

In R with ggplot2 this is `geom_ribbon()`, with `ymin` and `ymax` for the two edges. In Stata
it is `twoway rarea lower upper x, sort`.

## Arguments
- p: The plot to which the layer is added. Between them, the plot and the layer must map a
  variable or an expression to `x`. What the plot maps to `y` is not read.
- data: A frame for this layer alone, in place of the frame of the plot. It must hold what the
  layer takes from the plot. What the layer names itself is read in this frame.
- x: For this layer alone, the variable or the expression its bands are placed by along `x`,
  and ordered by.
- low: The variable, or the expression, that gives the lower edge of the band at each row.
- high: The same for the upper edge.
- fill: For this layer alone, as on `plot`: a bare name fills a band for each category of a
  variable, each in its colour, and a string sets one colour for every band. A band has one
  colour, so a number without value labels, which is shown on a ramp, is refused.
- group: The variable whose values part the rows into bands, for this layer alone, in place of
  `group:` of the plot. Any type will do.
- alpha: The opacity of the fill, from 0, which shows nothing, to 1, which hides what is under
  the band.

## Value
The `Plot` with the layer of bands on top. The plot given is not changed, so a plot bound to a
name can be extended in more than one way.

## Details
The axis. The two edges are positions along `y`: the `y` axis is wide enough for both, and
shares its scale with every other layer. Where the plot maps nothing to `y`, the axis takes
its title from `low:`.

The order. Within a group the rows are joined in the order of `x`, and rows with one value of
`x` in the order of the frame, as for `lines`.

Rows not drawn. A row with `NA` or an infinite value of `x` is left out. A row with `NA` or
an infinite value in either edge is left out too, and the band is broken there: it ends at the
row before and begins again at the row after. A note counts the rows left out, and another the
breaks. A row left with no neighbour fills nothing, and a note counts such rows.

The edges are drawn as they are given. Where `low:` is above `high:` the band is still filled
between the two, and nothing is said.

Groups. The rows are parted by every variable that `group:` and `fill:` name, together. Where
the plot maps a variable of categories to `color`, and the frame of the layer holds it, the
rows are parted by it as well, so that a band lies around each line of the plot.

The fill. Where neither the plot nor the layer says anything of `fill`, a band takes its
fill from `color:` of the plot: the colour of its category, where the frame of the layer
holds that variable, or the one colour the plot sets. `fill:` on the plot or on the layer
comes first. With nothing to take a fill from, a band is filled in the dark grey of the
theme's marks, which an `alpha:` of 0.3 makes a light grey on a white page. The key of a
legend is a patch of the fill, as see-through as the bands.

## See also
area, lines, plot

## Examples
> let fit = frame(educ = [8, 12, 16], wage = [9.1, 13.6, 18.2], se = [0.6, 0.3, 0.7])
> fit.plot(x: educ, y: wage).ribbon(low: wage - 2 * se, high: wage + 2 * se).lines()
plot · fit · wage against educ

  axis   title   scale    limits            breaks
  x      educ    linear   7.6 to 16.4       8, 10, 12, 14, 16
  y      wage    linear   7.315 to 20.185   8, 10, 12, 14, 16, 18, 20

  layer   mark     data   marks   settings
      1   ribbon   fit        3   low wage - 2 * se, high wage + 2 * se, alpha 0.3, 1 band
      2   lines    fit        3   width 1, 1 line
> fit.plot(x: educ, y: wage).ribbon(low: wage - se, high: wage + se, fill: "steelblue").points()
plot · fit · wage against educ

  axis   title   scale    limits          breaks
  x      educ    linear   7.6 to 16.4     8, 10, 12, 14, 16
  y      wage    linear   7.98 to 19.42   8, 10, 12, 14, 16, 18

  layer   mark     data   marks   settings
      1   ribbon   fit        3   low wage - se, high wage + se, fill steelblue, alpha 0.3, 1 band
      2   points   fit        3   size 4.5, alpha 1
