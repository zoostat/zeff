# dist.normal

## Summary
The normal distribution with a mean and a standard deviation

## In brief
- `dist.normal()`: the standard normal, with mean 0 and standard deviation 1.
- `dist.normal(mean: 100, sd: 15)`: with a mean and a standard deviation.

## Description
A `Distribution` value for the normal (Gaussian) distribution with mean `mean:` and standard
deviation `sd:`, both optional: `dist.normal()` is the standard normal. The value prints as its
family and parameters with the defaults filled in, and is passed to the methods `pdf`, `cdf`,
`quantile` and `draw`, which arrive with this release's distribution functions.

## Arguments
- mean: The mean, any finite number; 0 by default.
- sd: The standard deviation, a positive number; 1 by default. Zero, a negative number or `NA`
  is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)` the density, `cdf(x, upper:)` the distribution function
and its upper tail, `quantile(p)` the value at cumulative probability `p`, and `draw(n, seed:)` a
column of `n` draws.

## Details
The density is φ((x − μ)/σ)/σ with φ(z) = exp(−z²/2)/√(2π). The distribution function Φ is
computed from the complementary error function on each side, so that both tails keep their
relative accuracy down to 1e-300 (Engine and Project Specification, section 13: a tail is never
one minus the other side). The quantile is found by Halley steps on Φ from a rational start and is
exact at one half. Values are held to R and to an exact decimal reference within 1e-12 relative.

## A common mistake
Giving a standard deviation that is not positive. A variance, or a deviation worked out wrongly, may be zero or below; the parameter is a standard deviation above zero:

> dist.normal(mean: 0, sd: 0)
error[Z0413]: `sd` of `dist.normal` must be a positive number, but this is 0
  = help: `zeff help dist.normal` gives the domain of every parameter

Give the standard deviation, a positive number: `dist.normal(mean: 0, sd: 1)`. Where a variance is known, give its square root.

## Coming from R or Stata
| dist.normal(mean: m, sd: s).pdf(x) | dnorm(x, m, s) | normalden(x, m, s) |
| dist.normal(mean: m, sd: s).cdf(x) | pnorm(x, m, s) | normal((x - m) / s) |
| dist.normal(mean: m, sd: s).cdf(x, upper: true) | pnorm(x, m, s, lower.tail = FALSE) | normal(-(x - m) / s) |
| dist.normal(mean: m, sd: s).quantile(p) | qnorm(p, m, s) | m + s * invnormal(p) |
| dist.normal(mean: m, sd: s).draw(n) | rnorm(n, m, s) | rnormal(m, s) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. Stata's `normal()` and `invnormal()` are of the standard normal alone, so a mean and a standard deviation are worked in by hand. All three take the standard deviation, not the variance.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- For a positive quantity whose logarithm is normal, use `dist.lognormal`.
- For the tails of a mean from a small sample, use `dist.t`.
- To test whether data are normal, use `shapiro_test`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 13. Wiley.
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, sections 7.1
  and 26.2. National Bureau of Standards.

## See also
dist.lognormal, dist.t

## Examples
// The standard normal, when no parameter is given.
> dist.normal()
normal distribution · mean: 0, sd: 1

// A mean and a standard deviation: the score that nine tenths lie below, and the share below 130.
> let iq = dist.normal(mean: 100, sd: 15)
> iq.quantile(0.9)
> iq.cdf(130)
119.223
0.97725

// An upper tail, computed as a tail and not as one minus the rest.
> dist.normal().cdf(1.96, upper: true)
0.0249979

// The two-sided critical value at five per cent.
> dist.normal().quantile(0.975)
1.95996

// Three draws, the same under the same seed.
> dist.normal().draw(3, seed: 1)
Column<Float> · 3 values
1.22791
0.216564
0.445438
