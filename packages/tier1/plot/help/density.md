# density

## Summary
A density layer: the kernel estimate of the density of `x`, as a line

## Description
Estimates the density of the variable mapped to `x` and draws it as a line: a smooth curve
that is high where the values gather and low where they are few. `w.plot(x: wage).density()`
is the density of `wage`. It gives the plot back, so other layers and settings follow it in
the chain.

A density shows the shape of a distribution as a histogram does, without its bins: where a
histogram changes with where its bins begin, a density changes only with how much it
smooths, which is its bandwidth. Several densities are compared more easily in one panel
than several histograms, so a density suits a comparison of groups: colour the plot by the
group and each has its line.

In R this is `density()`, and with ggplot2 `geom_density()`. In Stata it is `kdensity`.

## Arguments
- p: The plot, which maps a variable of numbers to `x` and nothing to `y`.
- data: A frame for this layer alone, in place of the plot's. It holds the variable of `x`.
- bandwidth: The standard deviation of the kernel, in the units of `x`, above 0: how far the
  estimate smooths. Left out, it is Silverman's rule of thumb, from the values of each line.
- adjust: What the bandwidth is multiplied by, above 0. Above 1 the estimate is smoother, and
  below 1 it follows the values more closely.
- kernel: The shape that is centred on each value, a word without quotation marks:
  `gaussian`, the default, `epanechnikov`, `rectangular`, `triangular`, `biweight` or
  `cosine`.

## Value
The `Plot` with the layer added. The plot given is not changed.

## Details
The estimate. At a place `x` it is the mean, over the `n` values `x_i`, of a kernel `K`
centred on each: `(1 / n) Σ K(x - x_i)`. The kernel is a density itself, with mean zero and
with the bandwidth as its standard deviation, so the estimate is a density too: it is never
below zero and the area under it is one. It is computed at 512 places, from three bandwidths
below the smallest value to three above the largest, and the line is drawn through them.
Where one value lies very far from all the others the places are further apart than a kernel
is wide, and the line misses the peaks between them: set such a value aside with `where`, or
draw the logarithm of the variable.

The bandwidth. Silverman's rule of thumb is `0.9 A n^(-1/5)`, where `A` is the smaller of the
standard deviation and the interquartile range over 1.34: about the best bandwidth for values
from a normal distribution, and too smooth for a density with two peaks, which `adjust:`
below 1 shows. It is R's `bw.nrd0`. Where the values have no spread it falls back as R's
does. A small bandwidth draws every value as a bump and a large one hides a second peak: try
`adjust: 0.5` and `adjust: 2` before reading much into a shape.

The kernels. `gaussian` is the normal density, and reaches as far as the page. The others
are zero beyond a reach `a`: `rectangular`, level, with `a` the bandwidth times the square
root of 3; `triangular`, with the root of 6; `epanechnikov`, a parabola, with the root of 5;
`biweight`, the square of a parabola, with the root of 7; and `cosine`, one arch of a cosine.
Each is scaled so that the bandwidth is its standard deviation, as in R, so one bandwidth
smooths about alike whatever the kernel, and the kernel changes the estimate much less than
the bandwidth does. In Stata the bandwidth of a kernel is its reach, or a share of it, so
the same number there smooths differently.

What other programs do by default. R's `density` and ggplot2 use the Gaussian kernel and
Silverman's rule, as here; R bins the values first and so agrees with the exact estimate to
about three places. Stata's `kdensity` uses the Epanechnikov kernel and a bandwidth of its
own rule.

Groups and panels. Where the plot colours by categories each category has its line, with a
bandwidth and places of its own, and each is a density of area one, whatever the number of
its rows. In a plot parted into panels by `facet` each panel has the density of its rows.
A group with fewer than two values has no density, and a note says so.

Rows that are not used. A row with no value of `x`, or an infinite one, is left out, and a
note says how many there are.

The printed plot names the layer a density and says its kernel and its bandwidth.

## References
Silverman (1986), Density Estimation for Statistics and Data Analysis, Chapman and Hall,
London.
Rosenblatt (1956), Remarks on some nonparametric estimates of a density function, Annals of
Mathematical Statistics 27, 832–837.
Parzen (1962), On estimation of a probability density function and mode, Annals of
Mathematical Statistics 33, 1065–1076.

## See also
plot, histogram, lines, facet

## Examples
> let d = frame(score = [1, 2, 2, 3, 3, 3, 4, 4, 5, 9], arm = ["a", "b", "a", "b", "a", "b", "a", "b", "a", "b"])
> d.plot(x: score).density()
plot · d · score

  axis   title     scale    limits                   breaks
  x      score     linear   -1.84731 to 11.8473      0, 2, 4, 6, 8, 10
  y      density   linear   -0.0119175 to 0.263416   0, 0.05, 0.1, 0.15, 0.2, 0.25

  layer   mark      data   marks   settings
      1   density   d        512   gaussian kernel, Silverman's bandwidth, width 1, 1 line
> d.plot(x: score, color: arm).density(adjust: 0.5, kernel: epanechnikov)
plot · d · score

  axis   title     scale    limits                   breaks
  x      score     linear   -0.969905 to 10.2397     0, 2, 4, 6, 8, 10
  y      density   linear   -0.0275545 to 0.578645   0, 0.1, 0.2, 0.3, 0.4, 0.5

  aesthetic   title   scale      values   shown as
  color       arm     category   a, b     orange, sky blue

  layer   mark      data   marks   settings
      1   density   d      1,024   epanechnikov kernel, Silverman's bandwidth times 0.5, width 1, 2 lines
