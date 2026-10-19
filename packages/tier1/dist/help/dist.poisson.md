# dist.poisson

## Summary
Poisson distribution

## Description
A `Distribution` value for the Poisson distribution with mean `lambda:`: the number of events in a
fixed interval when they occur independently at a constant rate, the usual model for counts. Its
draws are whole numbers, so `draw` gives a `Column<Int>`; `pdf` is the probability of exactly `x`
events and is 0 at a value that is not a whole number. Like R's `pois` family (`dpois`, `ppois`,
`qpois`, `rpois`) and Stata's `poissonp()`, `poisson()`, `poissontail()` and `rpoisson()`.

## Arguments
- lambda: The mean, a positive number. Zero, a negative number or `NA` is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The probability of k events is e^−λ λ^k / k!, computed in Loader's saddle-point form
exp(−S(k) − D(k, λ)) / √(2πk) with the Stirling remainder and the deviance, so that it keeps every
digit for a mean of a thousand or more. The distribution function P(X ≤ k) is the upper regularised
incomplete gamma function Q(k + 1, λ), its own upper tail P(k + 1, λ) computed directly; `cdf` at a
value that is not a whole number counts the events up to the whole number below it. The quantile is
the smallest k with P(X ≤ k) ≥ p; `quantile(1)` is `Inf`.

## See also
dist.binomial, dist.negbinomial, dist.gamma

## References
- Loader, C. (2000). Fast and accurate computation of binomial probabilities. Technical report.
- Johnson, N. L., Kemp, A. W. and Kotz, S. (2005). *Univariate Discrete Distributions*, 3rd
  edition, chapter 4. Wiley.

## Examples
> let arrivals = dist.poisson(lambda: 2)
> arrivals
poisson distribution · lambda: 2
> arrivals.pdf(3)
0.180447
> arrivals.cdf(3, upper: true)
0.142877
> arrivals.quantile(0.9)
4
> dist.poisson(lambda: 4).draw(5, seed: 2)
Column<Int> · 5 values
3
8
1
8
4
