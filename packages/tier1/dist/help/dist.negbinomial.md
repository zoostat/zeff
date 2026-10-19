# dist.negbinomial

## Summary
Negative binomial distribution, by success probability or by its mean (NB2)

## Description
A `Distribution` value for the negative binomial distribution in either of its two forms: with
`size:` and `p:`, the number of failures before the `size:`-th success in trials that succeed with
probability `p:`; or with `mu:` and `size:`, the count with mean `mu:` and variance mu + mu²/size,
the NB2 form of count regression, where `size:` need not be a whole number and a larger `size:`
approaches the Poisson. The two forms are the same distribution with p = size / (size + mu); give
one or the other, not both (Z0410), and not neither (Z0406). Its draws are whole numbers, so `draw`
gives a `Column<Int>`. Like R's `nbinom` family with `prob =` or `mu =` and Stata's `nbinomialp()`,
`nbinomial()` and `nbinomialtail()`.

## Arguments
- size: The number of successes, or the dispersion parameter of the NB2 form; a positive number,
  not necessarily whole. Zero, a negative number or `NA` is refused (Z0413).
- p: The probability of success in each trial, above 0 and at most 1; refused outside (Z0413).
  `p: 1` is the point mass at zero failures.
- mu: The mean, a positive number, in place of `p:`; refused outside (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.
It prints in the form it was given.

## Details
The probability of k failures is Γ(k + size) / (Γ(size) k!) p^size (1 − p)^k, computed as
size / (size + k) times the binomial probability of `size` successes in k + size trials in
Loader's saddle-point form, which holds for a fractional `size:`. The distribution function
P(X ≤ k) is the regularised incomplete beta function I_p(size, k + 1), its upper tail computed
directly. The quantile is the smallest k with P(X ≤ k) ≥ p; `quantile(1)` is `Inf`.

## See also
dist.poisson, dist.binomial, dist.gamma

## References
- Johnson, N. L., Kemp, A. W. and Kotz, S. (2005). *Univariate Discrete Distributions*, 3rd
  edition, chapter 5. Wiley.
- Cameron, A. C. and Trivedi, P. K. (2013). *Regression Analysis of Count Data*, 2nd edition,
  section 3.3. Cambridge University Press.

## Examples
> let failures = dist.negbinomial(size: 3, p: 0.4)
> failures
negbinomial distribution · size: 3, p: 0.4
> failures.pdf(2)
0.13824
> failures.cdf(2)
0.31744
> let counts = dist.negbinomial(mu: 4.5, size: 3)
> counts
negbinomial distribution · mu: 4.5, size: 3
> counts.pdf(2)
0.13824
> counts.draw(5, seed: 2)
Column<Int> · 5 values
3
13
1
12
4
