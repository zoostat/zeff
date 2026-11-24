# dist.exponential

## Summary
The exponential distribution

## Description
A `Distribution` value for the exponential distribution with rate `rate:`, the waiting time
between events of a Poisson process with that rate; its mean is 1 / rate. The value prints as its
family and parameter with the default filled in.

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

## A common mistake
Giving a rate of zero, as when no event was seen:

> dist.exponential(rate: 0)
error[Z0413]: `rate` of `dist.exponential` must be a positive number, but this is 0
  = help: `zeff help dist.exponential` gives the domain of every parameter

The rate is above zero, and is one over the mean waiting time: a mean of 4 is `dist.exponential(rate: 0.25)`.

## Coming from R or Stata
| dist.exponential(rate: r).pdf(x) | dexp(x, r) | exponentialden(1 / r, x) |
| dist.exponential(rate: r).cdf(x) | pexp(x, r) | exponential(1 / r, x) |
| dist.exponential(rate: r).cdf(x, upper: true) | pexp(x, r, lower.tail = FALSE) | exponentialtail(1 / r, x) |
| dist.exponential(rate: r).quantile(p) | qexp(p, r) | invexponential(1 / r, p) |
| dist.exponential(rate: r).draw(n) | rexp(n, r) | rexponential(1 / r) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. R takes the rate, as here; Stata's functions take the scale, which is the mean, one over the rate.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- For the number of events in a period, use `dist.poisson`.
- For the waiting time for several events, use `dist.gamma`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 19. Wiley.

## See also
dist.gamma, dist.uniform

## Examples
// The waiting time when two events are expected in a unit of time.
> let waiting = dist.exponential(rate: 2)
> waiting
exponential distribution · rate: 2

// The density at one unit.
> waiting.pdf(1)
0.270671

// The probability of waiting no longer than one unit, and of waiting longer.
> waiting.cdf(1)
> waiting.cdf(1, upper: true)
0.864665
0.135335

// The median waiting time, below the mean of one half.
> waiting.quantile(0.5)
0.346574

// A rate of one, when none is given.
> dist.exponential()
exponential distribution · rate: 1
