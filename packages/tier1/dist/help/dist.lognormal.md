# dist.lognormal

## Summary
The log-normal distribution

## In brief
- `dist.lognormal()`: the exponential of a standard normal.
- `dist.lognormal(meanlog: 3, sdlog: 0.5)`: with the mean and the standard deviation of the logarithm.

## Description
A `Distribution` value for the log-normal distribution: the distribution of exp(Z) when Z is normal
with mean `meanlog:` and standard deviation `sdlog:`, both optional. It is the usual model for
wages, prices and other positive quantities whose logarithm is roughly symmetric; `wages.add(lwage =
log(wage))` and a normal fit to `lwage` is the same model written out. The value prints as its family
and parameters with the defaults filled in.

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

## A common mistake
Giving a standard deviation of the logarithm that is not positive:

> dist.lognormal(meanlog: 3, sdlog: -0.5)
error[Z0413]: `sdlog` of `dist.lognormal` must be a positive number, but this is -0.5
  = help: `zeff help dist.lognormal` gives the domain of every parameter

`sdlog:` is a standard deviation, above zero: `dist.lognormal(meanlog: 3, sdlog: 0.5)`.

## Coming from R or Stata
| dist.lognormal(meanlog: m, sdlog: s).pdf(x) | dlnorm(x, m, s) | normalden((ln(x) - m) / s) / (s * x) |
| dist.lognormal(meanlog: m, sdlog: s).cdf(x) | plnorm(x, m, s) | normal((ln(x) - m) / s) |
| dist.lognormal(meanlog: m, sdlog: s).quantile(p) | qlnorm(p, m, s) | exp(m + s * invnormal(p)) |
| dist.lognormal(meanlog: m, sdlog: s).draw(n) | rlnorm(n, m, s) | exp(rnormal(m, s)) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. The Stata forms are written through the normal distribution of the logarithm, which is what the log-normal is. The two parameters are the mean and the standard deviation of the logarithm, as in R, not of the quantity itself: its median is exp(meanlog), and its mean is larger.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- For the logarithm itself, use `dist.normal`.
- To cut the distribution at a bound, use `dist.truncated`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 14. Wiley.

## See also
dist.normal, dist.exponential, log

## Examples
// The parameters are those of the logarithm.
> let w = dist.lognormal(meanlog: 3, sdlog: 0.5)
> w
lognormal distribution · meanlog: 3, sdlog: 0.5

// The median is exp(meanlog).
> w.quantile(0.5)
20.0855

// The share below 20.
> w.cdf(20)
0.496595

// There is no density below zero.
> w.pdf(-1)
0

// Five draws, all positive.
> w.draw(5, seed: 3)
Column<Float> · 5 values
31.5428
24.5535
14.745
13.7531
28.9814
