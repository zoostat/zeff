# histogram

## Summary
A histogram of a variable of a frame: its values counted in bins

## Description
Counts the values of a variable in bins of one width and draws a bar for each bin, in one
call: `w.histogram(wage)`. It is the plot to begin with for one measured variable: where its
values lie, how far they spread, whether they lean to one side, have two peaks, or hold
values far from the rest.

It is a shorthand of the grammar: `w.histogram(wage, bins: 40)` is
`w.plot(x: wage).histogram(bins: 40)`, the same `Plot` value, and its options are those of
the layer. So everything a plot takes may follow it in the chain: `scale_x(log)`, `labels`,
`theme`, `export`.

`by:` gives each value of a variable a panel of its own, as `facet(by:)` does, with the same
bins in every panel, so that the groups can be compared.

In R this is `hist(x)`, and with ggplot2 `geom_histogram()`. In Stata it is `histogram x`.

## Arguments
- data: The frame whose rows are counted.
- var: The variable, or the expression, whose values are counted: a number.
- bins: The number of bins, or the rule that chooses it: `fd`, Freedman and Diaconis's;
  `sturges`; or `scott`.
- binwidth: The width of a bin, in the place of a number of bins.
- start: Where the first bin begins. Without it, at the smallest value.
- scale: What the height of a bar shows: `count`, the rows in the bin; `density`, so that the
  bars have area one; `fraction`, the share of the rows; or `percent`.
- by: A variable, or several in brackets, whose values part the plot into panels with the
  same axes.

## Value
A `Plot`, the one its grammar form gives. It is drawn where it is printed and may be bound,
extended and exported.

## Details
The bins. All bins have one width, and each holds its left edge and not its right, but for
the last, which holds both. The rule of Freedman and Diaconis takes the width
`2 IQR / n^(1/3)`, which follows the middle half of the values and is not stretched by a few
far ones; Sturges's takes `1 + log2(n)` bins, and Scott's the width `3.49 s / n^(1/3)`. The
numbers of bins are those R's `nclass.FD`, `nclass.Sturges` and `nclass.scott` give. A
histogram has a thousand bins at most.

The number of bins changes what is seen: few bins hide a second peak, many show noise. It is
worth drawing a histogram twice, with half and with twice the bins, before reading its shape.
A `density` shows the same distribution as a smooth line, and a `boxplot` compares many
groups in less room.

Rows not counted. A row with `NA` or an infinite value of `var` is left out, and a note says
how many there are.

The help page of the layer, `histogram` of a plot, says more of each option. A refusal of the
shorthand is the refusal of `plot`, of the layer or of `facet`, and points at the argument of
`histogram` that caused it.

## References
Freedman and Diaconis (1981), On the histogram as a density estimator: L2 theory, Zeitschrift
für Wahrscheinlichkeitstheorie und verwandte Gebiete 57, 453–476.
Sturges (1926), The choice of a class interval, Journal of the American Statistical
Association 21, 65–66.
Scott (1979), On optimal and data-based histograms, Biometrika 66, 605–610.

## See also
plot, density, boxplot, facet, summary

## Examples
> let d = frame(score = [1, 2, 2, 3, 3, 3, 4, 4, 5, 9])
> d.histogram(score)
plot · d · score

  axis   title   scale    limits          breaks
  x      score   linear   0.6 to 9.4      2, 4, 6, 8
  y      count   linear   -0.25 to 5.25   0, 1, 2, 3, 4, 5

  layer   mark        data   marks   settings
      1   histogram   d          5   5 bins of 1.6 from 1
