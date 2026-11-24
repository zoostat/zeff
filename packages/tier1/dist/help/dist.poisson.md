# dist.poisson

## Summary
Poisson distribution

## Description
A `Distribution` value for the Poisson distribution with mean `lambda:`: the number of events in a
fixed interval when they occur independently at a constant rate, the usual model for counts. Its
draws are whole numbers, so `draw` gives a `Column<Int>`; `pdf` is the probability of exactly `x`
events and is 0 at a value that is not a whole number.

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

## A common mistake
Giving a mean that is not positive:

> dist.poisson(lambda: -2)
error[Z0413]: `lambda` of `dist.poisson` must be a positive number, but this is -2
  = help: `zeff help dist.poisson` gives the domain of every parameter

`lambda:` is the mean number of events, above zero: `dist.poisson(lambda: 2)`.

## Coming from R or Stata
| dist.poisson(lambda: m).pdf(k) | dpois(k, m) | poissonp(m, k) |
| dist.poisson(lambda: m).cdf(k) | ppois(k, m) | poisson(m, k) |
| dist.poisson(lambda: m).cdf(k, upper: true) | ppois(k, m, lower.tail = FALSE) | poissontail(m, k + 1) |
| dist.poisson(lambda: m).quantile(q) | qpois(q, m) | no function of its own |
| dist.poisson(lambda: m).draw(n) | rpois(n, m) | rpoisson(m) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. The upper tail here, as in R, is the probability of more than `k` events; Stata's `poissontail(m, k)` is the probability of `k` or more, so the same tail is written with `k + 1`.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- For counts more spread than their mean, use `dist.negbinomial`.
- For the successes in a fixed number of trials, use `dist.binomial`.
- For the waiting time between the events, use `dist.exponential`.

## References
- Loader, C. (2000). Fast and accurate computation of binomial probabilities. Technical report.
- Johnson, N. L., Kemp, A. W. and Kotz, S. (2005). *Univariate Discrete Distributions*, 3rd
  edition, chapter 4. Wiley.

## See also
dist.binomial, dist.negbinomial, dist.gamma

## Examples
// The number of events when two are expected.
> let arrivals = dist.poisson(lambda: 2)
> arrivals
poisson distribution · lambda: 2

// The probability of exactly three.
> arrivals.pdf(3)
0.180447

// Of more than three.
> arrivals.cdf(3, upper: true)
0.142877

// The count that nine tenths of the periods stay at or below.
> arrivals.quantile(0.9)
4

// Draws are whole numbers.
> dist.poisson(lambda: 4).draw(5, seed: 2)
Column<Int> · 5 values
3
8
1
8
4
