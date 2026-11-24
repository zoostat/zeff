# dist.gamma

## Summary
Gamma distribution

## Description
A `Distribution` value for the gamma distribution with shape `shape:` and rate `rate:` (the
scale is 1 / rate): the distribution of a waiting time for `shape:` events of a Poisson process,
the conjugate prior of a Poisson rate, and the family the chi-square and exponential distributions
belong to.

## Arguments
- shape: The shape, a positive number. Zero, a negative number or `NA` is refused (Z0413).
- rate: The rate, a positive number; 1 by default. Refused likewise.

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density at x > 0 is λ (λx)^(α−1) e^(−λx) / Γ(α) and zero below, its prefactor computed in
the Stirling form so that a large shape loses no digits; the distribution function is the
regularised incomplete gamma function P(α, λx), its upper tail Q(α, λx) computed directly; the
quantile inverts P on the smaller tail and divides by the rate. Values are held to R and to an
exact decimal reference within 1e-12 relative, except for a shape above 100,000, where the
continued fraction reaches about 1e-10 (a known issue of this release).

## A common mistake
Giving the scale, as R's `dgamma(x, shape, scale = )` allows. The second parameter here is the rate:

> dist.gamma(shape: 2, scale: 0.5)
error[Z0404]: `dist.gamma` has no argument called `scale`
  = help: usage: `dist.gamma(shape, rate: Float = 1)`

The rate is one over the scale: `dist.gamma(shape: 2, rate: 2)`.

## Coming from R or Stata
| dist.gamma(shape: a, rate: r).pdf(x) | dgamma(x, a, r) | gammaden(a, 1 / r, 0, x) |
| dist.gamma(shape: a, rate: r).cdf(x) | pgamma(x, a, r) | gammap(a, r * x) |
| dist.gamma(shape: a, rate: r).cdf(x, upper: true) | pgamma(x, a, r, lower.tail = FALSE) | gammaptail(a, r * x) |
| dist.gamma(shape: a, rate: r).quantile(p) | qgamma(p, a, r) | invgammap(a, p) / r |
| dist.gamma(shape: a, rate: r).draw(n) | rgamma(n, a, r) | rgamma(a, 1 / r) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. R's third argument is the rate unless `scale =` is named; Stata's `gammap` and `invgammap` are of a scale of one, so the rate is worked in by hand, and its `gammaden` and `rgamma` take the scale.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- For a shape of one, the waiting time for one event, use `dist.exponential`.
- For a sum of squared normals, use `dist.chi2`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 17. Wiley.

## See also
dist.chi2, dist.exponential, dist.beta, gamma

## Examples
// A shape of 2 and a rate of 3: the mean is shape / rate.
> let g = dist.gamma(shape: 2, rate: 3)
> g
gamma distribution · shape: 2, rate: 3

// The density, the probability below and the median.
> g.pdf(0.5)
> g.cdf(0.5)
> g.quantile(0.5)
1.00409
0.442175
0.559449

// A rate of one, when none is given.
> dist.gamma(shape: 9).quantile(0.99)
17.4027
