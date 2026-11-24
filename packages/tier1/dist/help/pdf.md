# pdf

## Summary
Density, or probability for discrete distributions

## In brief
- `d.pdf(x)`: the density of a continuous distribution at `x`, or the probability of the count `x` of a discrete one.
- `d.pdf(column)`: the same for each row of a column.

## Description
The probability density of a continuous distribution at `x`, or the probability of the value `x`
for a discrete one (binomial, Poisson, negative binomial). Given a column, `pdf` applies row by row
and returns a column. `NA` gives `NA`.

## Arguments
- distribution: The distribution, a `Distribution` value such as `dist.normal()`.
- x: The point, an Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, and nothing is printed for it.

## Details
Each family's density is written out in closed form on the kernel's special functions and the
engine's pure-Rust elementary functions, so the result is the same on every target; the help page of
each family gives its formula. Outside a family's support the density is 0.

## A common mistake
Calling it with the value alone. The first thing it takes is the distribution, so the value is taken for it and `x` is found missing:

> pdf(0)
error[Z0406]: `pdf` needs `x`
  = help: usage: `distribution.pdf(x)`

Name the distribution: `dist.normal().pdf(0)`.

## Coming from R or Stata
| dist.normal().pdf(x) | dnorm(x) | normalden(x) |
| dist.exponential(rate: 2).pdf(x) | dexp(x, 2) | exponentialden(1 / 2, x) |
| dist.binomial(n: 10, p: 0.3).pdf(3) | dbinom(3, 10, 0.3) | binomialp(10, 3, 0.3) |

R has a `d` function for each family and Stata a function ending in `den`, or in `p` for a
count; here there is one method, and the distribution it is called on says which.

## Related tasks
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, use `quantile`.
- For random numbers, use `draw`.
- For a density estimated from data, use `density`.

## References
- The help page of each family (`zeff help dist.normal`, …) for its formula and reference.

## See also
cdf, quantile, draw

## Examples
// The height of the standard normal curve at its centre.
> dist.normal().pdf(0)
0.398942

// Of any distribution: the method is the same.
> dist.exponential(rate: 2).pdf(1)
0.270671

// For a count it is a probability: exactly three successes in ten trials.
> dist.binomial(n: 10, p: 0.3).pdf(3)
0.266828

// Row by row on a column; a row with `NA` stays `NA`.
> frame(z = [-1.96, 0, NA, 1.96]).add(density = dist.normal().pdf(z))
frame · 4 rows × 2 variables

          z     density
  1   -1.96   0.0584409
  2       0    0.398942
  3      NA          NA
  4    1.96   0.0584409
