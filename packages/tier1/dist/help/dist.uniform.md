# dist.uniform

## Summary
The uniform distribution on an interval

## In brief
- `dist.uniform()`: every value between 0 and 1 equally likely.
- `dist.uniform(min: 2, max: 5)`: between two ends.

## Description
A `Distribution` value for the continuous uniform distribution on [`min:`, `max:`], every value of
the interval equally likely; `dist.uniform()` is the standard uniform on [0, 1], the distribution
of the raw numbers every other draw is built from. The value prints as its family and parameters
with the defaults filled in.

## Arguments
- min: The lower end of the interval, any finite number; 0 by default.
- max: The upper end, a finite number greater than `min:`; 1 by default. A `max:` not above
  `min:` is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density is 1 / (max − min) on the interval and zero outside; the distribution function is
(x − min) / (max − min), clamped to [0, 1], its upper tail (max − x) / (max − min) computed on its
own; the quantile is min + p (max − min). A draw is min + (max − min) u for a uniform u of the
stream, so `dist.uniform().draw(n)` gives the stream's own numbers with 53 random bits each.

## A common mistake
Giving the two ends the wrong way round:

> dist.uniform(min: 5, max: 2)
error[Z0413]: `max` of `dist.uniform` must exceed `min`
  = help: `zeff help dist.uniform` gives the domain of every parameter

`max:` is above `min:`: `dist.uniform(min: 2, max: 5)`.

## Coming from R or Stata
| dist.uniform(min: a, max: b).pdf(x) | dunif(x, a, b) | no function of its own |
| dist.uniform(min: a, max: b).cdf(x) | punif(x, a, b) | no function of its own |
| dist.uniform(min: a, max: b).quantile(p) | qunif(p, a, b) | a + (b - a) * p |
| dist.uniform(min: a, max: b).draw(n) | runif(n, a, b) | runiform(a, b) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. Stata has the draws alone, `runiform()`, the rest being simple arithmetic.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- To pick rows of a frame at random, use `sample`.
- For a proportion that is not equally likely everywhere, use `dist.beta`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 26. Wiley.

## See also
dist.normal, dist.exponential

## Examples
// Every value between 2 and 5 equally likely.
> let u = dist.uniform(min: 2, max: 5)
> u
uniform distribution · min: 2, max: 5

// The density is one over the width.
> u.pdf(3)
0.333333

// A third of the interval lies below 3.
> u.cdf(3)
0.333333

// The first quartile.
> u.quantile(0.25)
2.75

// Three draws.
> u.draw(3, seed: 7)
Column<Float> · 3 values
4.86379
2.34253
3.22088

// The standard uniform on 0 to 1, when no end is given.
> dist.uniform()
uniform distribution · min: 0, max: 1
