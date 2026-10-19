# dist.beta

## Summary
Beta distribution

## Description
A `Distribution` value for the beta distribution on [0, 1] with shape parameters `a:` and `b:`:
the distribution of a proportion, the conjugate prior of a binomial probability, and the
distribution of order statistics of uniform draws. Its mean is a / (a + b). Like R's `beta`
family (`dbeta`, `pbeta`, `qbeta`, `rbeta`) and Stata's `ibeta()`, `ibetatail()` and
`invibeta()`.

## Arguments
- a: The first shape parameter, a positive number. Zero, a negative number or `NA` is refused
  (Z0413).
- b: The second shape parameter, a positive number, refused likewise.

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density on (0, 1) is x^(a−1) (1 − x)^(b−1) / B(a, b), computed in the form that keeps its
digits for large `a:` and `b:`, zero outside [0, 1], and at the ends infinite when the shape there
is below one; the distribution function is the regularised incomplete beta function I_x(a, b),
its upper tail computed directly; the quantile inverts I on the smaller tail. Values are held to
R and to an exact decimal reference within 1e-12 relative, except for a parameter above 100,000,
where the continued fraction reaches about 1e-10 (a known issue of this release).

## See also
dist.f, dist.gamma, dist.uniform, beta

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 25. Wiley.

## Examples
> let share = dist.beta(a: 2, b: 3)
> share
beta distribution · a: 2, b: 3
> share.pdf(0.4)
1.728
> share.cdf(0.4)
0.5248
> share.quantile(0.9)
0.679539
