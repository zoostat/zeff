# qq_plot

## Summary
A quantile-quantile plot: a variable against the quantiles of a distribution

## Description
Draws each value of a variable against the value a distribution would put at its rank, with a
reference line: `w.qq_plot(wage)` compares the wages with a normal distribution. Where the
points follow the line the variable has the shape of the distribution; where they leave it
they show how it differs. It is the plot to look at before a method that assumes normal
errors, and the companion of `shapiro_test` and `ks_test`, which give a p-value and no
picture.

Without a distribution the comparison is with the normal. Any distribution of `dist` may be
given: `r.qq_plot(resid, dist.t(df: 5))` for heavier tails, `d.qq_plot(wait,
dist.exponential(rate: 1))` for waiting times.

`by:` gives each value of a variable a panel of its own, as `facet(by:)` does, each with the
pairs and the line of its own rows.

In R this is `qqnorm(x)` with `qqline(x)`, and for another distribution `qqplot` against its
quantiles. In Stata it is `qnorm x`.

## Arguments
- data: The frame whose rows are drawn.
- var: The variable, or the expression, whose values are compared: a number.
- distribution: The distribution the values are compared with, as `dist` makes it. Without
  it, the normal with mean 0 and standard deviation 1.
- by: A variable, or several in brackets, whose values part the plot into panels with the
  same axes.

## Value
A `Plot` of two layers, the points and the reference line. Along `x` are the quantiles of the
distribution, along `y` the values of the variable. It is drawn where it is printed and may
be bound, extended and exported.

## Details
The pairs. The `n` values are put in order, and the value of rank `i` is paired with the
quantile of the distribution at the plotting position `(i - a) / (n + 1 - 2a)`, where `a` is
3/8 for ten values or fewer and 1/2 for more: R's `ppoints`, as `qqnorm` uses it. Values that
are equal take ranks next to one another. Stata's `qnorm` takes `i / (n + 1)`, which differs
a little at the two ends.

The line passes through the first and the third quartiles of the values and of the
distribution, as R's `qqline` does. It follows the middle half of the data and is not pulled
by the tails, which are what the plot is looked at for. Against the standard normal its slope
estimates the standard deviation and its height at 0 the mean, so the mean and the spread of
the variable need not be given: only the shape is compared.

Reading it. Points that bend up at the right and down at the left have heavier tails than the
distribution; the other way, lighter ones. A curve, up at both ends, is skewness to the
right. One or two points far from the line at an end are values to look at one by one. With
few values the points wander from the line by chance alone: twenty values from a normal
distribution seldom lie on it.

How it is built. The frame drawn is the frame given with one variable more, `theoretical`,
the quantile of each row, so a point is still its row. The plot is
`plot(x: theoretical, y: var).points()` and a layer of `lines` for the reference line, which
has a small frame of its own with the two ends of the line. A frame that already has a
variable called `theoretical` is refused.

Rows not drawn. A row with `NA` or an infinite value of `var` has no pair and is left out.
The note that counts such rows names `theoretical` beside the variable, since the row lacks
both. A group with fewer than two values has no line, and a note counts such groups.

## References
Wilk and Gnanadesikan (1968), Probability plotting methods for the analysis of data,
Biometrika 55, 1–17.
Blom (1958), Statistical Estimates and Transformed Beta-Variables, Wiley, New York.

## See also
shapiro_test, ks_test, histogram, density, plot

## Examples
> let d = frame(score = [1, 2, 2, 3, 3, 3, 4, 4, 5, 9])
> d.qq_plot(score)
plot · d · score against theoretical

  axis   title                                 scale    limits              breaks
  x      Quantiles of normal(mean: 0, sd: 1)   linear   -1.7013 to 1.7013   -1.5, -1, -0.5, 0, 0.5, 1, 1.5
  y      score                                 linear   0.6 to 9.4          2, 4, 6, 8

  layer   mark     data            marks   settings
      1   points   d                  10   size 4.5, alpha 1
      2   lines    its own frame       2   y line, width 1, 1 line
> d.qq_plot(score, dist.t(df: 3))
plot · d · score against theoretical

  axis   title                   scale    limits                breaks
  x      Quantiles of t(df: 3)   linear   -2.35306 to 2.35306   -2, -1, 0, 1, 2
  y      score                   linear   0.261814 to 9.4161    2, 4, 6, 8

  layer   mark     data            marks   settings
      1   points   d                  10   size 4.5, alpha 1
      2   lines    its own frame       2   y line, width 1, 1 line
