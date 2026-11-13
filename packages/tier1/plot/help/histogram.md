# histogram

## Summary
A histogram layer: the rows counted in bins along `x`

## Description
Parts the range of the variable mapped to `x` into bins of one width and draws a bar over
each bin, as high as the number of rows whose value falls in it. `w.plot(x: wage).histogram()`
is the histogram of `wage`. It gives the plot back, so other layers and settings follow it in
the chain.

A histogram shows the shape of a distribution: where the values gather, how far they spread,
whether there is one peak or two, whether one tail is longer. What it shows depends on the
bins, so try another number of them before reading much into a bump or a gap.

In R this is `hist()`, and with ggplot2 `geom_histogram()`. In Stata it is `histogram`.

## Arguments
- p: The plot, which maps a variable of numbers to `x` and nothing to `y`.
- data: A frame for this layer alone, in place of the plot's. It holds the variable of `x`.
- bins: How many bins: a whole number, 1 or more, or a rule that gives the number from the
  values, a word without quotation marks: `sturges`, `scott`, or `fd`, the rule of Freedman
  and Diaconis, which is the default.
- binwidth: The width of a bin in the units of `x`, above 0. Where it is given the bins are
  that wide and `bins:` is not read.
- start: Where the first bin begins. Left out, it is the smallest value. It may not be above
  the smallest value.
- scale: What the height of a bar shows, a word without quotation marks: `count`, the number
  of rows in the bin, the default; `density`, so that the bars have area one; `fraction`, the
  share of the rows; `percent`, that share times a hundred.

## Value
The `Plot` with the layer added. The plot given is not changed.

## Details
The bins. They begin at `start:` and are of one width. With `binwidth:` there are as many as
reach the largest value. Without it there are as many as `bins:` says, and each is as wide as
it takes for them to reach from the start to the largest value exactly. A bin holds its left
edge and not its right, so a value on an edge is counted in the bin that begins there; the
last bin holds its right edge too, which is the largest value. Values that are all one value
have one bin, one unit wide, with the value in its middle.

The rules. Each gives a number of bins from the values, as R's `nclass.Sturges`,
`nclass.scott` and `nclass.FD` do. `sturges`: one more than the logarithm of the number of
rows to the base two, rounded up, which suits about a normal shape and too few bins for a
large or skewed sample. `scott`: bins 3.5 standard deviations over the cube root of the
number of rows wide. `fd`: bins twice the interquartile range over the cube root of the
number of rows wide, which a few far values do not stretch as they stretch a standard
deviation; where the interquartile range is zero, it is taken between quantiles further out.

What other programs do by default. R's `hist` uses Sturges's rule and then moves the edges
of the bins to round numbers, so its bins are not those of this layer even where their number
is the same. ggplot2 draws 30 bins and says so. Stata uses the smaller of the square root of
the number of rows and ten times its logarithm to the base ten, and scales the bars as a
density. Here the default is the rule of Freedman and Diaconis and counts.

The scales. With `n` rows counted, `c` of them in a bin of width `h`: `count` is `c`,
`density` is `c / (n h)`, `fraction` is `c / n`, and `percent` is `100 c / n`. Under `density`
the areas of the bars sum to one, so that a density curve can be drawn over them; under
`fraction` the heights sum to one, and under `percent` to a hundred. The axis is named by the
scale.

Groups and panels. Where the plot fills by categories, each bin has a bar for each category,
one on another, the first category on top; `n` is then the number of rows of all the
categories, so that the stack is the histogram of them all. In a plot parted into panels
by `facet` the bins are those of all the rows, the same in every panel, and `n` is the number
of rows of the panel.

Rows that are not counted. A row with no value of `x`, or an infinite one, is in no bin, and
a note says how many there are.

The most bins. A histogram has a thousand bins at most: more are not told apart on a page. A
number of bins or a width that asks for more is refused; a rule that asks for more, as it may
where one value lies far from all the others, is given a thousand, and a note says so.

The printed plot names the layer a histogram and says its bins: how many, how wide, and
where the first begins. The counts themselves are read off the figure.

## References
Sturges (1926), The choice of a class interval, Journal of the American Statistical
Association 21, 65–66.
Scott (1979), On optimal and data-based histograms, Biometrika 66, 605–610.
Freedman and Diaconis (1981), On the histogram as a density estimator: L2 theory, Zeitschrift
für Wahrscheinlichkeitstheorie und verwandte Gebiete 57, 453–476.

## See also
plot, bars, facet, scale

## Examples
> let d = frame(score = [1, 2, 2, 3, 3, 3, 4, 4, 5, 9], arm = ["a", "b", "a", "b", "a", "b", "a", "b", "a", "b"])
> d.plot(x: score).histogram()
plot · d · score

  axis   title   scale    limits          breaks
  x      score   linear   0.6 to 9.4      2, 4, 6, 8
  y      count   linear   -0.25 to 5.25   0, 1, 2, 3, 4, 5

  layer   mark        data   marks   settings
      1   histogram   d          5   5 bins of 1.6 from 1
> d.plot(x: score).histogram(binwidth: 2, start: 0, scale: percent)
plot · d · score

  axis   title     scale    limits         breaks
  x      score     linear   -0.5 to 10.5   0, 2, 4, 6, 8, 10
  y      percent   linear   -2.5 to 52.5   0, 10, 20, 30, 40, 50

  layer   mark        data   marks   settings
      1   histogram   d          5   5 bins of 2 from 0
> d.plot(x: score, fill: arm).histogram(bins: 4)
plot · d · score

  axis   title   scale    limits          breaks
  x      score   linear   0.6 to 9.4      2, 4, 6, 8
  y      count   linear   -0.25 to 5.25   0, 1, 2, 3, 4, 5

  aesthetic   title   scale      values   shown as
  fill        arm     category   a, b     orange, sky blue

  layer   mark        data   marks   settings
      1   histogram   d          8   4 bins of 2 from 1
