# dist.exponential

## Summary
The exponential distribution

## Description
A `Distribution` value for the exponential distribution with rate `rate:`, the waiting time
between events of a Poisson process with that rate; its mean is 1 / rate. The value prints as its
family and parameter with the default filled in. Like R's `exp` family (`dexp`, `pexp`, `qexp`,
`rexp`) and Stata's `exponential()`, `exponentialden()` and `invexponential()`, which take the
scale 1 / rate.

## Arguments
- rate: The rate, a positive number; 1 by default. Zero, a negative number or `NA` is refused
  (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density at x ≥ 0 is λ exp(−λx) and zero below; the distribution function is 1 − exp(−λx),
computed as −expm1(−λx) so that small probabilities keep their digits, and its upper tail
exp(−λx) directly; the quantile is −log1p(−p) / λ. Draws are the inverse CDF of the uniform
stream.

## See also
dist.gamma, dist.uniform

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 19. Wiley.

## Examples
> let waiting = dist.exponential(rate: 2)
> waiting
exponential distribution · rate: 2
> waiting.pdf(1)
0.270671
> waiting.cdf(1)
0.864665
> waiting.cdf(1, upper: true)
0.135335
> waiting.quantile(0.5)
0.346574
