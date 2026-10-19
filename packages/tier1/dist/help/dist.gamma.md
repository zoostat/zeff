# dist.gamma

## Summary
Gamma distribution

## Description
A `Distribution` value for the gamma distribution with shape `shape:` and rate `rate:` (the
scale is 1 / rate): the distribution of a waiting time for `shape:` events of a Poisson process,
the conjugate prior of a Poisson rate, and the family the chi-square and exponential distributions
belong to. Like R's `gamma` family with `rate =` (`dgamma`, `pgamma`, `qgamma`, `rgamma`) and
Stata's `gammap()`, `gammaptail()` and `invgammap()`, which take the shape alone.

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

## See also
dist.chi2, dist.exponential, dist.beta, gamma

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 17. Wiley.

## Examples
> let g = dist.gamma(shape: 2, rate: 3)
> g
gamma distribution · shape: 2, rate: 3
> g.pdf(0.5)
1.00409
> g.cdf(0.5)
0.442175
> g.quantile(0.5)
0.559449
> dist.gamma(shape: 9).quantile(0.99)
17.4027
