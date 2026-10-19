# dist.binomial

## Summary
Binomial distribution

## Description
A `Distribution` value for the binomial distribution: the number of successes in `n:` independent
trials, each succeeding with probability `p:`. Its draws are whole numbers, so `draw` gives a
`Column<Int>`; `pdf` is the probability of exactly `x` successes and is 0 at a value that is not a
whole number. Like R's `binom` family (`dbinom`, `pbinom`, `qbinom`, `rbinom`) and Stata's
`binomialp()`, `binomial()`, `binomialtail()` and `rbinomial()`.

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

## See also
dist.poisson, dist.negbinomial, choose

## References
- Loader, C. (2000). Fast and accurate computation of binomial probabilities. Technical report.
- Johnson, N. L., Kemp, A. W. and Kotz, S. (2005). *Univariate Discrete Distributions*, 3rd
  edition, chapter 3. Wiley.

## Examples
> let coins = dist.binomial(n: 10, p: 0.3)
> coins
binomial distribution · n: 10, p: 0.3
> coins.pdf(3)
0.266828
> coins.cdf(3)
0.649611
> coins.quantile(0.5)
3
> dist.binomial(n: 20, p: 0.5).draw(5, seed: 2)
Column<Int> · 5 values
10
14
7
14
10
