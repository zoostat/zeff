# dist.negbinomial

## Summary
Negative binomial distribution, by success probability or by its mean (NB2)

## In brief
- `dist.negbinomial(size: 3, p: 0.4)`: the failures before the third success, each trial succeeding with probability 0.4.
- `dist.negbinomial(mu: 4.5, size: 3)`: the count with mean 4.5 and variance mu + mu²/size, the form of count regression.

## Description
A `Distribution` value for the negative binomial distribution in either of its two forms: with
`size:` and `p:`, the number of failures before the `size:`-th success in trials that succeed with
probability `p:`; or with `mu:` and `size:`, the count with mean `mu:` and variance mu + mu²/size,
the NB2 form of count regression, where `size:` need not be a whole number and a larger `size:`
approaches the Poisson. The two forms are the same distribution with p = size / (size + mu); give
one or the other, not both (Z0410), and not neither (Z0406). Its draws are whole numbers, so `draw`
gives a `Column<Int>`.

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

## A common mistake
Giving both forms at once, the success probability and the mean. The two say the same thing in two ways, and one is given:

> dist.negbinomial(size: 3, p: 0.4, mu: 4.5)
error[Z0410]: `dist.negbinomial` takes `p:` or `mu:`, not both
  = help: `size:` with `p:` counts failures before the size-th success; `mu:` with `size:` is the mean parameterisation (NB2)

Give `size:` with `p:`, or `mu:` with `size:`: `dist.negbinomial(mu: 4.5, size: 3)`.

## Coming from R or Stata
| dist.negbinomial(size: n, p: p).pdf(k) | dnbinom(k, n, p) | nbinomialp(n, k, p) |
| dist.negbinomial(size: n, p: p).cdf(k) | pnbinom(k, n, p) | nbinomial(n, k, p) |
| dist.negbinomial(mu: m, size: n).pdf(k) | dnbinom(k, size = n, mu = m) | nbinomialp(n, k, n / (n + m)) |
| dist.negbinomial(size: n, p: p).draw(r) | rnbinom(r, n, p) | rnbinomial(n, p) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. R takes either form, with `prob =` or `mu =`, as here. Stata's functions take the probability, so the mean form is written with p = size / (size + mu).

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- For counts whose variance equals their mean, use `dist.poisson`.
- For the successes in a fixed number of trials, use `dist.binomial`.

## References
- Johnson, N. L., Kemp, A. W. and Kotz, S. (2005). *Univariate Discrete Distributions*, 3rd
  edition, chapter 5. Wiley.
- Cameron, A. C. and Trivedi, P. K. (2013). *Regression Analysis of Count Data*, 2nd edition,
  section 3.3. Cambridge University Press.

## See also
dist.poisson, dist.binomial, dist.gamma

## Examples
// By the success probability: the failures before the third success.
> let failures = dist.negbinomial(size: 3, p: 0.4)
> failures
> failures.pdf(2)
> failures.cdf(2)
negbinomial distribution · size: 3, p: 0.4
0.13824
0.31744

// By the mean, the form of count regression; the same distribution, with p = size / (size + mu).
> let counts = dist.negbinomial(mu: 4.5, size: 3)
> counts
> counts.pdf(2)
negbinomial distribution · mu: 4.5, size: 3
0.13824

// Draws are whole numbers, more spread than a Poisson's of the same mean.
> counts.draw(5, seed: 2)
Column<Int> · 5 values
3
13
1
12
4
