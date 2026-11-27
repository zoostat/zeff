# dist.hypergeometric

## Summary
Hypergeometric distribution

## Description
A `Distribution` value for the hypergeometric distribution: the number of successes among
`draws:` items taken without replacement from a population of `successes:` items of one kind and
`failures:` of the other. It is the distribution behind Fisher's exact test and behind sampling
from a finite population. Its draws are whole numbers, so `draw` gives a `Column<Int>`; `pdf` is
the probability of exactly `x` successes and is 0 at a value that is not a whole number.

## Arguments
- successes: The number of items of the kind counted, a whole number of 0 or more.
- failures: The number of items of the other kind, a whole number of 0 or more. The population
  is the two together.
- draws: The number of items taken, a whole number of 0 or more and at most the population.

A negative or fractional number, `NA`, or more draws than the population holds is refused (Z0413
or Z0401).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
With m successes, n failures and d draws, the probability of k successes is
C(m, k) C(n, d − k) / C(m + n, d), for k from max(0, d − n) to min(d, m), and its mean is
d m / (m + n). It is computed as a quotient of three binomial probabilities in Loader's
saddle-point form, so that no digit is lost to the logarithm of a binomial coefficient in a
population of a thousand million. The distribution function is summed directly: the tail on the
side of the mean where the value lies is added up outwards from it, each term from the one before,
and the other tail is its complement. A tail far out is therefore a sum of a few positive terms
and keeps its digits however small it is. `cdf` at a value that is not a whole number counts the
successes up to the whole number below it. The quantile is the smallest k with P(X ≤ k) ≥ p;
`quantile(0)` and `quantile(1)` are the ends of the range.

For a large population the quantile differs from R's, and R is the one in error: its `qhyper`
adds the probabilities up from the lower end of the range and, by the upper tail, has lost what
it needs. For a million draws from five hundred million of each kind it gives a million, the end
of the range, at a probability of 1 − 1e-10; R's own distribution function shows the answer to be
503,179, which is what `quantile` gives.

One point differs from R by design. R's `phyper` rounds a value within 1e-7 below zero up to
zero and answers the probability of no success, where its `pbinom` and `ppois` answer zero. Here
every count distribution answers zero at a negative value, since no count lies at or below it.

Values are held to exact rational arithmetic within 1e-12 relative for populations up to a few
thousand, and to R up to a population of a thousand million. Drawing from a very large
population is slow, since each draw inverts a sum of thousands of terms.

## A common mistake
Asking for more draws than the population holds, often by giving the size of the whole
population where the number of failures belongs:

> dist.hypergeometric(successes: 10, failures: 7, draws: 20)
error[Z0413]: `draws` of `dist.hypergeometric` must be at most `successes` plus `failures`: the draws are made without replacement
  = help: `zeff help dist.hypergeometric` gives the domain of every parameter

The second number is the count of the other kind, not the total: a population of 30 of which 10
are successes is `dist.hypergeometric(successes: 10, failures: 20, draws: 20)`.

## Coming from R or Stata
| dist.hypergeometric(successes: m, failures: n, draws: d).pdf(k) | dhyper(k, m, n, d) | hypergeometricp(m + n, m, d, k) |
| dist.hypergeometric(successes: m, failures: n, draws: d).cdf(k) | phyper(k, m, n, d) | hypergeometric(m + n, m, d, k) |
| dist.hypergeometric(successes: m, failures: n, draws: d).cdf(k, upper: true) | phyper(k, m, n, d, lower.tail = FALSE) | 1 - hypergeometric(m + n, m, d, k) |
| dist.hypergeometric(successes: m, failures: n, draws: d).quantile(q) | qhyper(q, m, n, d) | no function of its own |
| dist.hypergeometric(successes: m, failures: n, draws: d).draw(r) | rhyper(r, m, n, d) | rhypergeometric(m + n, m, d) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. R's arguments are the same three in the same order, under the names `m`, `n` and `k` of white balls, black balls and balls drawn. Stata's functions take the size of the whole population first, then the number of successes in it, then the draws, and the value last; Stata has no upper tail of its own, and one minus the lower tail loses the digits of a small one.

## Related tasks
- For the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- To test the association in a two-by-two table exactly, use `fisher_test`.
- For draws made with replacement, or from a population too large to matter, use `dist.binomial`.
- To take rows of a frame without replacement, use `sample`.

## References
- Johnson, N. L., Kemp, A. W. and Kotz, S. (2005). *Univariate Discrete Distributions*, 3rd
  edition, chapter 6. Wiley.
- Loader, C. (2000). Fast and accurate computation of binomial probabilities. Technical report.

## See also
dist.binomial, fisher_test, sample, choose

## Examples
// Eight items taken from ten of one kind and seven of the other.
> let h = dist.hypergeometric(successes: 10, failures: 7, draws: 8)
> h
hypergeometric distribution · successes: 10, failures: 7, draws: 8

// The probability of exactly five successes.
> h.pdf(5)
0.362814

// Of five or fewer, and of more than five.
> h.cdf(5)
> h.cdf(5, upper: true)
0.782188
0.217812

// The median, and the ends of the range: at least one success, since only seven are failures.
> h.quantile(0.5)
> h.quantile(0)
> h.quantile(1)
5
1
8

// A thousand draws from a population of a thousand million, half of them successes.
> dist.hypergeometric(successes: 500000000, failures: 500000000, draws: 1000).cdf(470)
0.0310115
