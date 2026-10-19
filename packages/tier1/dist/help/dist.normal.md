# dist.normal

## Summary
The normal distribution with a mean and a standard deviation

## Description
A `Distribution` value for the normal (Gaussian) distribution with mean `mean:` and standard
deviation `sd:`, both optional: `dist.normal()` is the standard normal. The value prints as its
family and parameters with the defaults filled in, and is passed to the methods `pdf`, `cdf`,
`quantile` and `draw`, which arrive with this release's distribution functions. Like R's `norm`
family of functions and Stata's `normal()`, `normalden()` and `invnormal()`, with the parameters
named rather than positional.

## Arguments
- mean: The mean, any finite number; 0 by default.
- sd: The standard deviation, a positive number; 1 by default. Zero, a negative number or `NA`
  is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density is φ((x − μ)/σ)/σ with φ(z) = exp(−z²/2)/√(2π). The distribution function Φ is
computed from the complementary error function on each side, so that both tails keep their
relative accuracy down to 1e-300 (Engine and Project Specification, section 13: a tail is never
one minus the other side). The quantile is found by Halley steps on Φ from a rational start and is
exact at one half. Values are held to R and to an exact decimal reference within 1e-12 relative.

## See also
dist.lognormal, dist.t

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 13. Wiley.
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, sections 7.1
  and 26.2. National Bureau of Standards.

## Examples
> dist.normal()
normal distribution · mean: 0, sd: 1
> let iq = dist.normal(mean: 100, sd: 15)
> iq
normal distribution · mean: 100, sd: 15
