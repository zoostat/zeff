# dist.truncated

## Summary
A distribution truncated to an interval, exact in every method

## Description
A `Distribution` value for another distribution restricted to the interval [`lower:`, `upper:`]:
the values outside are dropped and the probability inside is scaled back to one, which is the
distribution of the variable given that it fell in the interval. Either bound may be left open
(`-Inf` and `Inf` by default). It takes any distribution, a discrete one included, and another
truncation or a mixture. The value prints as the inner distribution with its bounds. Like the
`truncdist` package of R (`dtrunc`, `ptrunc`, `qtrunc`, `rtrunc`) and Stata's `rnormal()` with
rejection written by hand, which Zeff never needs.

## Arguments
- d: The distribution to truncate, a `Distribution` value such as `dist.normal()`.
- lower: The lower end of the interval, a number or `-Inf`; `-Inf` by default. `NA` is refused
  (Z0413).
- upper: The upper end, a number or `Inf` above `lower:`; `Inf` by default. A bound not above
  `lower:`, or an interval holding no probability of the distribution, is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
With Z = P(lower ≤ X ≤ upper), the density on the interval is f(x) / Z and zero outside; the
distribution function is (F(x) − F(lower⁻)) / Z and its upper tail (S(x) − S(upper)) / Z, each
from the inner distribution's tail on the same side, so that a far tail keeps its digits; the
quantile is the inner quantile at F(lower⁻) + p Z, held inside the interval. For a discrete
distribution the interval is closed at both ends, so the mass at `lower:` is kept (the `truncdist`
package drops it: its interval is open below). Values near the centre of the inner distribution
subtract two probabilities of similar size and keep about eleven digits. A draw is the inverse CDF
of one uniform number of the stream, so it is exact and never rejects.

## See also
dist.mixture, dist.normal, dist.poisson

## References
- Nadarajah, S. and Kotz, S. (2006). R programs for truncated distributions. *Journal of
  Statistical Software*, 16, code snippet 2.
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, section 13.10. Wiley.

## Examples
> let tn = dist.truncated(dist.normal(), lower: -1, upper: 2)
> tn
normal distribution · mean: 0, sd: 1 · truncated to [-1, 2]
> tn.cdf(0.5)
0.65088
> tn.quantile(1)
2
> let tp = dist.truncated(dist.poisson(lambda: 3), lower: 1, upper: 6)
> tp
poisson distribution · lambda: 3 · truncated to [1, 6]
> tp.pdf(1)
0.162933
> tp.draw(4, seed: 3)
Column<Int> · 4 values
4
4
2
2
