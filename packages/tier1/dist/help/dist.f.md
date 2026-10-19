# dist.f

## Summary
F distribution

## Description
A `Distribution` value for the F distribution with `df1:` and `df2:` degrees of freedom: the
distribution of a ratio of two independent chi-square variables each divided by its degrees of
freedom, and so of the F statistics of analysis of variance and of tests of several regression
coefficients at once. Like R's `f` family (`df`, `pf`, `qf`, `rf`) and Stata's `F()`, `Ftail()`
and `invF()`.

## Arguments
- df1: The numerator degrees of freedom, a positive number. Zero, a negative number or `NA` is
  refused (Z0413).
- df2: The denominator degrees of freedom, a positive number, refused likewise.

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
With w = d₁x / (d₁x + d₂), the distribution function is the regularised incomplete beta function
I_w(d₁/2, d₂/2), its upper tail computed directly; the density is the beta density at w times
dw/dx = d₁d₂ / (d₁x + d₂)², so that large degrees of freedom lose no digits; the quantile inverts
the incomplete beta function on the smaller tail and gives x = (d₂/d₁) w/(1 − w). At zero the
density is infinite below two numerator degrees of freedom, one at exactly two, zero above. A
p-value is the upper tail: `dist.f(df1: 3, df2: 20).cdf(3.1, upper: true)`.

## See also
dist.t, dist.chi2, dist.beta

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 27. Wiley.
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 26.6.
  National Bureau of Standards.

## Examples
> let f = dist.f(df1: 3, df2: 20)
> f
f distribution · df1: 3, df2: 20
> f.quantile(0.95)
3.09839
> f.cdf(3.1, upper: true)
0.0499245
> f.pdf(1)
0.430854
