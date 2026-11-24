# dist.chi2

## Summary
Chi-square distribution

## Description
A `Distribution` value for the chi-square distribution with `df:` degrees of freedom: the
distribution of a sum of `df:` squared standard normals, and so of likelihood-ratio, Wald and
Pearson test statistics under their null hypotheses. It is the gamma distribution with shape
`df:`/2 and rate ½.

## Arguments
- df: The degrees of freedom, a positive number (not necessarily whole). Zero, a negative number
  or `NA` is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density at x > 0 is x^(ν/2 − 1) e^(−x/2) / (2^(ν/2) Γ(ν/2)) and zero below; the distribution
function is the regularised incomplete gamma function P(ν/2, x/2), its upper tail Q(ν/2, x/2)
computed directly, so that the p-value of a large statistic keeps its digits; the quantile inverts
P on the smaller tail. A p-value is the upper tail: `dist.chi2(df: 5).cdf(11.07, upper: true)`.

## A common mistake
Giving degrees of freedom that are not positive:

> dist.chi2(df: -1)
error[Z0413]: `df` of `dist.chi2` must be a positive number, but this is -1
  = help: `zeff help dist.chi2` gives the domain of every parameter

The degrees of freedom are above zero: `dist.chi2(df: 5)`.

## Coming from R or Stata
| dist.chi2(df: v).pdf(x) | dchisq(x, v) | chi2den(v, x) |
| dist.chi2(df: v).cdf(x) | pchisq(x, v) | chi2(v, x) |
| dist.chi2(df: v).cdf(x, upper: true) | pchisq(x, v, lower.tail = FALSE) | chi2tail(v, x) |
| dist.chi2(df: v).quantile(p) | qchisq(p, v) | invchi2(v, p) |
| dist.chi2(df: v).draw(n) | rchisq(n, v) | rchi2(v) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. The p-value of a chi-square statistic is the upper tail, `cdf(x, upper: true)`, Stata's `chi2tail`.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- To test a table of counts, use `chi2_test`.
- For the family it belongs to, use `dist.gamma`.
- For a ratio of two of them, use `dist.f`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 18. Wiley.
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 26.4.
  National Bureau of Standards.

## See also
dist.gamma, dist.f, dist.normal

## Examples
// The chi-square distribution with five degrees of freedom.
> let chi5 = dist.chi2(df: 5)
> chi5
chi2 distribution · df: 5

// The critical value at five per cent.
> chi5.quantile(0.95)
11.0705

// The p-value of a statistic: the upper tail.
> chi5.cdf(11.07, upper: true)
0.0500096

// The density at a value.
> chi5.pdf(3)
0.15418
