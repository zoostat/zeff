# dist.beta

## Summary
Beta distribution

## Description
A `Distribution` value for the beta distribution on [0, 1] with shape parameters `a:` and `b:`:
the distribution of a proportion, the conjugate prior of a binomial probability, and the
distribution of order statistics of uniform draws. Its mean is a / (a + b).

## Arguments
- a: The first shape parameter, a positive number. Zero, a negative number or `NA` is refused
  (Z0413).
- b: The second shape parameter, a positive number, refused likewise.

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density on (0, 1) is x^(a−1) (1 − x)^(b−1) / B(a, b), computed in the form that keeps its
digits for large `a:` and `b:`, zero outside [0, 1], and at the ends infinite when the shape there
is below one; the distribution function is the regularised incomplete beta function I_x(a, b),
its upper tail computed directly; the quantile inverts I on the smaller tail. Values are held to
R and to an exact decimal reference within 1e-12 relative, except for a parameter above 100,000,
where the continued fraction reaches about 1e-10 (a known issue of this release).

## A common mistake
Giving a shape parameter of zero:

> dist.beta(a: 0, b: 3)
error[Z0413]: `a` of `dist.beta` must be a positive number, but this is 0
  = help: `zeff help dist.beta` gives the domain of every parameter

Both shapes are above zero: `dist.beta(a: 1, b: 3)`.

## Coming from R or Stata
| dist.beta(a: a, b: b).pdf(x) | dbeta(x, a, b) | betaden(a, b, x) |
| dist.beta(a: a, b: b).cdf(x) | pbeta(x, a, b) | ibeta(a, b, x) |
| dist.beta(a: a, b: b).cdf(x, upper: true) | pbeta(x, a, b, lower.tail = FALSE) | ibetatail(a, b, x) |
| dist.beta(a: a, b: b).quantile(p) | qbeta(p, a, b) | invibeta(a, b, p) |
| dist.beta(a: a, b: b).draw(n) | rbeta(n, a, b) | rbeta(a, b) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. Stata names the distribution function after the incomplete beta function it is, `ibeta`.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- For a count of successes with a known probability, use `dist.binomial`.
- For every value of an interval equally likely, use `dist.uniform`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 25. Wiley.

## See also
dist.f, dist.gamma, dist.uniform, beta

## Examples
// A proportion with mean a / (a + b), here 0.4.
> let share = dist.beta(a: 2, b: 3)
> share
beta distribution · a: 2, b: 3

// The density, the probability below and the ninth decile.
> share.pdf(0.4)
> share.cdf(0.4)
> share.quantile(0.9)
1.728
0.5248
0.679539

// Three draws, each between 0 and 1.
> share.draw(3, seed: 5)
Column<Float> · 3 values
0.555626
0.833119
0.299388
