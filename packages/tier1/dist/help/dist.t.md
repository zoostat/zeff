# dist.t

## Summary
Student's t distribution

## Description
A `Distribution` value for Student's t distribution with `df:` degrees of freedom: the
distribution of a sample mean's standardised deviation under normality, and so of the t
statistics of regression coefficients and of the one- and two-sample t tests. Its tails are
heavier than the normal's and approach it as `df:` grows. Like R's `t` family (`dt`, `pt`, `qt`,
`rt`) and Stata's `t()`, `ttail()` and `invt()`.

## Arguments
- df: The degrees of freedom, a positive number (not necessarily whole). Zero, a negative number,
  `Inf` or `NA` is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density is (1 + t²/ν)^(−(ν+1)/2) / (√ν B(½, ν/2)), computed in logarithms with the beta
function of the kernel, which keeps its digits for large ν. The distribution function is half the
regularised incomplete beta function I_x(ν/2, ½) at x = ν/(ν + t²), on the side of t, so that a
far tail such as P(T > 30) with three degrees of freedom keeps its relative accuracy; above 400,000
degrees of freedom the normal approximation R uses, z = t(1 − 1/4ν)/√(1 + t²/2ν), gives both the
same value as R and the accuracy the continued fraction would lose there. The quantile inverts the
incomplete beta function on the smaller tail. A two-sided p-value is twice the upper tail of |t|:
`2 * dist.t(df: 24).cdf(2.5, upper: true)`.

## See also
dist.normal, dist.f, dist.chi2

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 28. Wiley.
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 26.7.
  National Bureau of Standards.

## Examples
> let t24 = dist.t(df: 24)
> t24
t distribution · df: 24
> t24.quantile(0.975)
2.0639
> t24.cdf(2.064, upper: true)
0.0249948
> dist.t(df: 3).cdf(-30)
4.06764e-05
