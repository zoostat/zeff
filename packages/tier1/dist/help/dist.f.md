# dist.f

## Summary
F distribution

## Description
A `Distribution` value for the F distribution with `df1:` and `df2:` degrees of freedom: the
distribution of a ratio of two independent chi-square variables each divided by its degrees of
freedom, and so of the F statistics of analysis of variance and of tests of several regression
coefficients at once.

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

## A common mistake
Giving one of the two degrees of freedom. An F has those of its numerator and those of its denominator:

> dist.f(df1: 3)
error[Z0406]: `dist.f` needs `df2`
  = help: usage: `dist.f(df1, df2)`

Give both: `dist.f(df1: 3, df2: 20)`.

## Coming from R or Stata
| dist.f(df1: a, df2: b).pdf(x) | df(x, a, b) | Fden(a, b, x) |
| dist.f(df1: a, df2: b).cdf(x) | pf(x, a, b) | F(a, b, x) |
| dist.f(df1: a, df2: b).cdf(x, upper: true) | pf(x, a, b, lower.tail = FALSE) | Ftail(a, b, x) |
| dist.f(df1: a, df2: b).quantile(p) | qf(p, a, b) | invF(a, b, p) |
| dist.f(df1: a, df2: b).draw(n) | rf(n, a, b) | no function of its own |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. The p-value of an F statistic is the upper tail, `cdf(x, upper: true)`, Stata's `Ftail`.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, a critical value, use `quantile`.
- For random numbers, use `draw`.
- For an analysis of variance, use `anova`.
- To test several coefficients of a fit at once, use `test`.
- For the distribution of each of the two sums, use `dist.chi2`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 27. Wiley.
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 26.6.
  National Bureau of Standards.

## See also
dist.t, dist.chi2, dist.beta

## Examples
// An F with 3 and 20 degrees of freedom.
> let f = dist.f(df1: 3, df2: 20)
> f
f distribution · df1: 3, df2: 20

// The critical value at five per cent.
> f.quantile(0.95)
3.09839

// The p-value of a statistic: the upper tail.
> f.cdf(3.1, upper: true)
0.0499245

// The density at a value.
> f.pdf(1)
0.430854
