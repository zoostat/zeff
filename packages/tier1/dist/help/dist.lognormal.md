# dist.lognormal

## Summary
The log-normal distribution

## Description
A `Distribution` value for the log-normal distribution: the distribution of exp(Z) when Z is normal
with mean `meanlog:` and standard deviation `sdlog:`, both optional. It is the usual model for
wages, prices and other positive quantities whose logarithm is roughly symmetric; `wages.add(lwage =
log(wage))` and a normal fit to `lwage` is the same model written out. The value prints as its family
and parameters with the defaults filled in. Like R's `lnorm` family and Stata's `lognormal()`,
`lognormalden()` and `invlognormal()`.

## Arguments
- meanlog: The mean of the logarithm, any finite number; 0 by default.
- sdlog: The standard deviation of the logarithm, a positive number; 1 by default. Zero, a
  negative number or `NA` is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density at x > 0 is φ((ln x − μ)/σ) / (xσ) and zero at or below zero; the distribution
function is Φ((ln x − μ)/σ), both tails from `erfc` on their own side; the quantile is
exp(μ + σ Φ⁻¹(p)); the median is exp(μ) and the mean exp(μ + σ²/2). Draws are the inverse CDF of the
uniform stream. Values are held to R within 1e-12 relative.

## See also
dist.normal, dist.exponential, log

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 14. Wiley.

## Examples
> let w = dist.lognormal(meanlog: 3, sdlog: 0.5)
> w
lognormal distribution · meanlog: 3, sdlog: 0.5
> w.quantile(0.5)
20.0855
> w.cdf(20)
0.496595
> w.pdf(-1)
0
