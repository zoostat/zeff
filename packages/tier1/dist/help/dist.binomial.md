# dist.binomial

## Summary
Binomial distribution

## Description
A `Distribution` value for the binomial distribution: the number of successes in `n:` independent
trials, each succeeding with probability `p:`. Its draws are whole numbers, so `draw` gives a
`Column<Int>`; `pdf` is the probability of exactly `x` successes and is 0 at a value that is not a
whole number.

## Arguments
- n: The number of trials, a whole number of 0 or more. A negative or fractional `n` is refused
  (Z0413 or Z0401).
- p: The probability of success, between 0 and 1. Outside that or `NA` is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The probability of k successes is computed in Loader's saddle-point form, exp(S(n) − S(k) − S(n − k)
− D) / √(2πk(n − k)/n), with the Stirling remainders S and the deviance D = k ln(k/np) + (n − k)
ln((n − k)/n(1 − p)) evaluated by series near the mean, so that every digit survives for a million
trials where the logarithm of the binomial coefficient would not. The distribution function P(X ≤ k)
is the regularised incomplete beta function I_{1−p}(n − k, k + 1), its upper tail computed directly;
`cdf` at a value that is not a whole number counts the successes up to the whole number below it.
The quantile is the smallest k with P(X ≤ k) ≥ p, found from the normal approximation by a bracket
and halving; `quantile(1)` is `n`.

## A common mistake
Giving the probability as a percentage:

> dist.binomial(n: 10, p: 30)
error[Z0413]: `p` of `dist.binomial` must be a probability between 0 and 1, but this is 30
  = help: `zeff help dist.binomial` gives the domain of every parameter

`p:` is a probability between 0 and 1: `dist.binomial(n: 10, p: 0.3)`.

## Coming from R or Stata
| dist.binomial(n: n, p: p).pdf(k) | dbinom(k, n, p) | binomialp(n, k, p) |
| dist.binomial(n: n, p: p).cdf(k) | pbinom(k, n, p) | binomial(n, k, p) |
| dist.binomial(n: n, p: p).cdf(k, upper: true) | pbinom(k, n, p, lower.tail = FALSE) | binomialtail(n, k + 1, p) |
| dist.binomial(n: n, p: p).quantile(q) | qbinom(q, n, p) | no function of its own |
| dist.binomial(n: n, p: p).draw(m) | rbinom(m, n, p) | rbinomial(n, p) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. The upper tail here, as in R, is the probability of more than `k` successes; Stata's `binomialtail(n, k, p)` is the probability of `k` or more, so the same tail is written with `k + 1`.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- To test a proportion, use `proportion_test`.
- For counts with no fixed number of trials, use `dist.poisson`.
- For the number of failures before a given number of successes, use `dist.negbinomial`.

## References
- Loader, C. (2000). Fast and accurate computation of binomial probabilities. Technical report.
- Johnson, N. L., Kemp, A. W. and Kotz, S. (2005). *Univariate Discrete Distributions*, 3rd
  edition, chapter 3. Wiley.

## See also
dist.poisson, dist.negbinomial, choose

## Examples
// The successes in ten trials that each succeed three times in ten.
> let coins = dist.binomial(n: 10, p: 0.3)
> coins
binomial distribution · n: 10, p: 0.3

// The probability of exactly three.
> coins.pdf(3)
0.266828

// Of three or fewer, and of more than three.
> coins.cdf(3)
> coins.cdf(3, upper: true)
0.649611
0.350389

// The median number of successes.
> coins.quantile(0.5)
3

// Draws are whole numbers.
> dist.binomial(n: 20, p: 0.5).draw(5, seed: 2)
Column<Int> · 5 values
10
14
7
14
10
