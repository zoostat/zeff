# dist.weibull

## Summary
Weibull distribution

## Description
A `Distribution` value for the Weibull distribution with shape `shape:` and scale `scale:`: the
usual model of a time to failure, whose hazard falls with time for a shape below one, is constant
at a shape of one (the exponential) and rises above it. The scale is the value below which 63.2
per cent of the distribution lies, whatever the shape.

## Arguments
- shape: The shape, a positive number. Zero, a negative number or `NA` is refused (Z0413).
- scale: The scale, a positive number, in the units of the variable; 1 by default. Refused
  likewise.

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
With z = x / λ, the density at x > 0 is (k / λ) z^(k−1) e^(−z^k) and zero below; at zero it is
unbounded for a shape below one, 1 / λ at a shape of one and zero above it. The distribution
function is 1 − e^(−z^k), its upper tail e^(−z^k) computed directly, and the quantile is
λ (−log(1 − p))^(1/k). The mean is λ Γ(1 + 1/k). Values are held to R within 1e-12 relative, at
shapes and scales from the everyday to 1e7 and 1e12.

Where the two differ, R is the one in error: far beyond the scale at a very large shape, R's
`dweibull` multiplies a power that has overflowed by an exponential that has underflowed and
gives `NaN`; the density there is zero, and zero is what `pdf` gives.

## A common mistake
Giving a rate, as `dist.exponential` and `dist.gamma` take. The second parameter here is the
scale, as in R and Stata:

> dist.weibull(shape: 2, rate: 0.5)
error[Z0404]: `dist.weibull` has no argument called `rate`
  = help: usage: `dist.weibull(shape, scale: Float = 1)`

A Weibull distribution with a shape of one and a scale of λ is the exponential distribution with
rate 1 / λ: `dist.weibull(shape: 2, scale: 2)` is what was meant.

## Coming from R or Stata
| dist.weibull(shape: k, scale: s).pdf(x) | dweibull(x, k, s) | weibullden(k, s, x) |
| dist.weibull(shape: k, scale: s).cdf(x) | pweibull(x, k, s) | weibull(k, s, x) |
| dist.weibull(shape: k, scale: s).cdf(x, upper: true) | pweibull(x, k, s, lower.tail = FALSE) | weibulltail(k, s, x) |
| dist.weibull(shape: k, scale: s).quantile(p) | qweibull(p, k, s) | invweibull(k, s, p) |
| dist.weibull(shape: k, scale: s).draw(n) | rweibull(n, k, s) | rweibull(k, s) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. All three take the shape and then the scale. Stata's `streg` reports a Weibull model by the logarithm of its shape and by coefficients on the scale of the hazard, which are not these two parameters.

## Related tasks
- For the density, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, use `quantile`.
- For random numbers, use `draw`.
- For a constant hazard, a shape of one, use `dist.exponential`.
- For a waiting time for several events, use `dist.gamma`.
- For a lifetime observed only within a window, use `dist.truncated`.

## References
- Weibull, W. (1951). A statistical distribution function of wide applicability. *Journal of
  Applied Mechanics*, 18, 293–297.
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1994). *Continuous Univariate Distributions*,
  volume 1, 2nd edition, chapter 21. Wiley.

## See also
dist.exponential, dist.gamma, dist.lognormal, dist.truncated

## Examples
// A shape of 1.5 and a scale of 2: a hazard that rises with time.
> let w = dist.weibull(shape: 1.5, scale: 2)
> w
weibull distribution · shape: 1.5, scale: 2

// The density, the probability below and the median.
> w.pdf(1)
> w.cdf(1)
> w.quantile(0.5)
0.372392
0.297811
1.56644

// The share still running at the scale is e^(−1), whatever the shape.
> w.cdf(2, upper: true)
0.367879

// A scale of one, when none is given.
> dist.weibull(shape: 0.5).quantile(0.9)
5.3019
