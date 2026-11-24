# dist.t

## Summary
Student's t distribution

## Description
A `Distribution` value for Student's t distribution with `df:` degrees of freedom: the
distribution of a sample mean's standardised deviation under normality, and so of the t
statistics of regression coefficients and of the one- and two-sample t tests. Its tails are
heavier than the normal's and approach it as `df:` grows.

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

## A common mistake
Giving degrees of freedom of zero, as a sample of one observation would have:

> dist.t(df: 0)
error[Z0413]: `df` of `dist.t` must be a positive number, but this is 0
  = help: `zeff help dist.t` gives the domain of every parameter

The degrees of freedom are above zero; for a mean of n observations they are n − 1: `dist.t(df: 24)` for 25.

## Coming from R or Stata
| dist.t(df: v).pdf(x) | dt(x, v) | tden(v, x) |
| dist.t(df: v).cdf(x) | pt(x, v) | t(v, x) |
| dist.t(df: v).cdf(x, upper: true) | pt(x, v, lower.tail = FALSE) | ttail(v, x) |
| dist.t(df: v).quantile(p) | qt(p, v) | invt(v, p) |
| dist.t(df: v).draw(n) | rt(n, v) | rt(v) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. Stata's `invttail(v, p)` takes the probability above the value; `quantile` takes the probability below, as R's `qt` does.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- To test a mean, or the difference of two, use `t_test`.
- For the limit as the degrees of freedom grow, use `dist.normal`.
- For the square of a t, an F with one degree of freedom above, use `dist.f`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 28. Wiley.
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 26.7.
  National Bureau of Standards.

## See also
dist.normal, dist.f, dist.chi2

## Examples
// Student's t with 24 degrees of freedom.
> let t24 = dist.t(df: 24)
> t24
t distribution · df: 24

// The two-sided critical value at five per cent, beside the normal's 1.96.
> t24.quantile(0.975)
2.0639

// The upper tail beyond it.
> t24.cdf(2.064, upper: true)
0.0249948

// A far tail keeps its digits.
> dist.t(df: 3).cdf(-30)
4.06764e-05

// Degrees of freedom need not be whole, as Welch's are not.
> dist.t(df: 17.4).quantile(0.975)
2.10613
