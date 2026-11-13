# density

## Summary
A density plot of a variable of a frame: a smooth line over its values

## Description
Draws the kernel estimate of the density of a variable as a line, in one call:
`w.density(wage)`. It shows what a histogram shows, where the values lie and how they spread,
without the steps of the bins, and several groups can be laid over one another.

It is a shorthand of the grammar: `w.density(wage, color: union)` is
`w.plot(x: wage, color: union).density()`, the same `Plot` value, and its options are those
of the layer. So everything a plot takes may follow it in the chain: `scale_x(log)`,
`labels`, `theme`, `export`.

`color:` draws a line for each category of a variable, each in its colour, with a legend.
`by:` gives each value of a variable a panel of its own, as `facet(by:)` does.

In R this is `plot(density(x))`, and with ggplot2 `geom_density()`. In Stata it is
`kdensity x`.

## Arguments
- data: The frame whose rows are drawn.
- var: The variable, or the expression, whose density is estimated: a number.
- color: A variable shown as categories; a line for each of them, each in its colour, each
  the density of its own rows.
- bandwidth: The bandwidth of the kernel, its standard deviation, above 0. Without it,
  Silverman's rule of thumb chooses it from the values.
- adjust: A number the bandwidth is multiplied by, above 0: 0.5 for a line half as smooth,
  2 for one twice as smooth.
- kernel: The shape of the kernel: `gaussian`, `epanechnikov`, `rectangular`, `triangular`,
  `biweight` or `cosine`.
- by: A variable, or several in brackets, whose values part the plot into panels with the
  same axes.

## Value
A `Plot`, the one its grammar form gives. It is drawn where it is printed and may be bound,
extended and exported.

## Details
The estimate. At a place `x` the estimate is the mean, over the values, of a kernel centred
on each: a small bump for every value, added up. The bandwidth is the width of the bump, and
it matters far more than the shape of the kernel: a small one follows every value and shows
noise, a large one smooths a second peak away. Silverman's rule of thumb takes
`0.9 min(s, IQR / 1.34) n^(-1/5)`, R's `bw.nrd0`, which suits one peak and oversmooths two;
`adjust:` below 1 is the first thing to try where two peaks are suspected.

The line is taken at 512 places, from three bandwidths below the smallest value to three
above the largest, so it reaches beyond the data: for a variable that cannot be below zero,
such as a wage, the line may run a little below zero all the same. Each group of `color:`,
and each panel, has the density of its own rows with its own bandwidth, and its area is one.
A group with fewer than two values has no density, and a note counts such groups.

Rows not used. A row with `NA` or an infinite value of `var` is left out, and a note says how
many there are.

The help page of the layer, `density` of a plot, says more of each option. A refusal of the
shorthand is the refusal of `plot`, of the layer or of `facet`, and points at the argument of
`density` that caused it.

## References
Silverman (1986), Density Estimation for Statistics and Data Analysis, Chapman and Hall,
London.
Sheather and Jones (1991), A reliable data-based bandwidth selection method for kernel
density estimation, Journal of the Royal Statistical Society B 53, 683–690.

## See also
plot, histogram, boxplot, facet, summary

## Examples
> let d = frame(score = [1, 2, 2, 3, 3, 3, 4, 4, 5, 9])
> d.density(score)
plot · d · score

  axis   title     scale    limits                   breaks
  x      score     linear   -1.84731 to 11.8473      0, 2, 4, 6, 8, 10
  y      density   linear   -0.0119175 to 0.263416   0, 0.05, 0.1, 0.15, 0.2, 0.25

  layer   mark      data   marks   settings
      1   density   d        512   gaussian kernel, Silverman's bandwidth, width 1, 1 line
