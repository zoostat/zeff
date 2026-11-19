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
distribution function is the probability between `lower:` and x over Z, and its upper tail the
probability between x and `upper:` over Z. The probability between two points is the difference
of the inner distribution's tails on the side where both are small: the lower tails for two
points below the median, the upper tails for two points above it. An interval far out in either
tail therefore keeps its probability and its digits. Where the two tails are still nearly equal (a
point close to a bound, or a narrow interval), their difference would keep few digits, and the
density of a continuous distribution is integrated between the two points instead; the result
then keeps thirteen digits (held against the series of that integral for the normal and
Student's t, and against the closed form for the exponential). The studentized range is the
exception: its density is itself a sum, and its truncation keeps the difference of the tails.
One limit is that of the numbers themselves: a probability of the inner distribution below
2.2e-308 is held with fewer digits, and below 4.9e-324 it is zero. A truncation that lies
wholly that far out (a normal distribution cut beyond 37 standard deviations, an exponential
beyond 700 mean lifetimes) is exact near its bound and loses its digits further in: for a
standard normal cut at −37, the distribution function agrees with R's on the log scale to nine
digits down to −37.8, is 5 per cent off at −38.4 and is 0 from −38.5, and the quantile below a
probability of about 1e-24 is the last point the tail can still tell. No software that holds
probabilities and not their logarithms does better there.
The quantile is the root of that distribution function, searched from the inner quantile at
F(lower⁻) + p Z, on the lower tail up to one half and on the upper tail beyond; beside a bound
that is not zero it is the double nearest the root. For a discrete distribution the interval is
closed at both ends, so the mass at `lower:` is kept (the `truncdist` package drops it: its
interval is open below), and the quantile is the smallest whole number whose probability at or
below reaches p. A discrete distribution keeps the whole numbers of the interval, from the
smallest at or above `lower:` to the largest at or below `upper:`, and a bound within 1e-7 of a
whole number counts as that number, as the count distributions read their own argument: with
`lower: 1e-10, upper: 0.5` the truncation is the single value 0, and with `lower: 0.2` it is
refused, since no whole number lies between 0.2 and 0.5. Its quantile at 0 is the first number
kept and its quantile at 1 the last, or the largest the distribution can give where that is
smaller; a bound that is not a whole number is never returned. A draw is the inverse CDF of one
uniform number of the stream, so it is exact and never rejects.

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
> let far = dist.truncated(dist.normal(), lower: 9)
> far.quantile(0.5)
9.07579
> dist.truncated(dist.normal(), lower: 0, upper: 2).cdf(1.196288e-12)
1e-12
