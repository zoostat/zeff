# dist.chi2

## Summary
Chi-square distribution

## Description
A `Distribution` value for the chi-square distribution with `df:` degrees of freedom: the
distribution of a sum of `df:` squared standard normals, and so of likelihood-ratio, Wald and
Pearson test statistics under their null hypotheses. It is the gamma distribution with shape
`df:`/2 and rate ½. Like R's `chisq` family (`dchisq`, `pchisq`, `qchisq`, `rchisq`) and Stata's
`chi2()`, `chi2tail()` and `invchi2()`.

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

## See also
dist.gamma, dist.f, dist.normal

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 18. Wiley.
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 26.4.
  National Bureau of Standards.

## Examples
> let chi5 = dist.chi2(df: 5)
> chi5
chi2 distribution · df: 5
> chi5.quantile(0.95)
11.0705
> chi5.cdf(11.07, upper: true)
0.0500096
> chi5.pdf(3)
0.15418
