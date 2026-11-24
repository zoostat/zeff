# cdf

## Summary
P(X ≤ x); upper: true gives P(X > x) accurately in the tail

## In brief
- `d.cdf(x)`: the probability of a value at or below `x`.
- `d.cdf(x, upper: true)`: the probability of a value above `x`, the upper tail, computed as a tail.
- `d.cdf(column)`: the same for each row of a column.

## Description
The cumulative probability P(X ≤ x) of a distribution at `x`, or with `upper: true` the upper tail
P(X > x), computed on its own side so that a tail probability of 1e-300 keeps its digits (Engine
and Project Specification, section 13: a tail is never one minus the other). Given a column, `cdf`
applies row by row and returns a column. `NA` gives `NA`.

## Arguments
- distribution: The distribution, a `Distribution` value such as `dist.t(df: 24)`.
- x: The point, an Int or a Float, or a column of either.
- upper: `true` for P(X > x), the upper tail; `false` (the default) for P(X ≤ x).

## Value
A Float between 0 and 1; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, and nothing is printed for it.

## Details
For a discrete distribution P(X ≤ x) sums the probabilities up to the largest whole number not
above `x`, and `upper: true` is the complementary sum from the next whole number. A p-value is a
tail: `dist.normal().cdf(2.5, upper: true)` is the one-sided p-value of z = 2.5, and twice it the
two-sided one.

## A common mistake
Asking for the upper tail in R's way, by saying that the lower one is not wanted. The option names the tail that is:

> dist.normal().cdf(1.96, lower: false)
error[Z0404]: `cdf` has no argument called `lower`
  = help: usage: `distribution.cdf(x, upper: Bool = false)`

Write `dist.normal().cdf(1.96, upper: true)`.

## Coming from R or Stata
| dist.normal().cdf(x) | pnorm(x) | normal(x) |
| dist.normal().cdf(x, upper: true) | pnorm(x, lower.tail = FALSE) | normal(-x) |
| dist.t(df: v).cdf(x, upper: true) | pt(x, v, lower.tail = FALSE) | ttail(v, x) |
| dist.chi2(df: v).cdf(x, upper: true) | pchisq(x, v, lower.tail = FALSE) | chi2tail(v, x) |

A p-value written as one minus the lower probability, `1 - pnorm(x)`, loses its digits far in
the tail. `lower.tail = FALSE` in R, the functions ending in `tail` in Stata, and `upper: true`
here compute the tail itself.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For the value at a given probability, use `quantile`.
- For random numbers, use `draw`.
- For a test that gives the p-value with its statistic, use `t_test`.

## References
- The help page of each family (`zeff help dist.normal`, …) for its formula and reference.

## See also
pdf, quantile, draw

## Examples
// The probability below 1.96 under the standard normal.
> dist.normal().cdf(1.96)
0.975002

// The upper tail beyond it.
> dist.normal().cdf(1.96, upper: true)
0.0249979

// Far in the tail the upper probability keeps its digits, where one minus the lower would be 0.
> dist.normal().cdf(10, upper: true)
> 1 - dist.normal().cdf(10)
7.61985e-24
0

// A two-sided p-value of a t statistic of 2.3 on 24 degrees of freedom.
> 2 * dist.t(df: 24).cdf(2.3, upper: true)
0.0304496

// Row by row on a column.
> frame(z = [-1.96, 0, 1.96]).add(below = dist.normal().cdf(z))
frame · 3 rows × 2 variables

          z       below
  1   -1.96   0.0249979
  2       0         0.5
  3    1.96    0.975002
