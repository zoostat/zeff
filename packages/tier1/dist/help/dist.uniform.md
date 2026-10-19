# dist.uniform

## Summary
The uniform distribution on an interval

## Description
A `Distribution` value for the continuous uniform distribution on [`min:`, `max:`], every value of
the interval equally likely; `dist.uniform()` is the standard uniform on [0, 1], the distribution
of the raw numbers every other draw is built from. The value prints as its family and parameters
with the defaults filled in. Like R's `unif` family and Stata's `runiform()`.

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

## See also
dist.normal, dist.exponential

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 26. Wiley.

## Examples
> let u = dist.uniform(min: 2, max: 5)
> u
uniform distribution · min: 2, max: 5
> u.pdf(3)
0.333333
> u.cdf(3)
0.333333
> u.quantile(0.25)
2.75
> u.draw(3, seed: 7)
Column<Float> · 3 values
4.86379
2.34253
3.22088
