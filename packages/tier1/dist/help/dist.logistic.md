# dist.logistic

## Summary
The logistic distribution with a centre and a scale

## In brief
- `dist.logistic()`: the standard logistic, with centre 0 and scale 1.
- `dist.logistic(location: 100, scale: 15)`: with a centre and a scale.

## Description
A `Distribution` value for the logistic distribution with centre `location:` and spread `scale:`:
a symmetric, bell-shaped distribution with heavier tails than the normal, whose distribution
function is the logistic curve. It is the distribution of the error behind a logistic regression,
and its quantile function is the log-odds.

## Arguments
- location: The centre, which is the mean and the median, a finite number; 0 by default. An
  infinite value or `NA` is refused (Z0413).
- scale: The spread, a positive number; 1 by default. It is not the standard deviation, which is
  the scale times π / √3, about 1.814 times the scale. Zero, a negative number or `NA` is refused
  (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
With z = (x − μ) / s, the density is e^(−z) / (s (1 + e^(−z))²), the distribution function
1 / (1 + e^(−z)) and the quantile μ + s log(p / (1 − p)). Each tail is computed directly from
the exponential that is at most one, so that a small tail keeps its digits. Values are held to R
within 1e-12 relative, at locations and scales from the everyday to 1e7 and 1e12.

Where the two differ, R is the one in error. Its `qlogis` takes the logarithm of the odds as
they stand, and close to a probability of one half, where the odds are close to one, it keeps
fewer digits the closer the probability is: four at 0.5 + 1e-12. Here the quantile near one half
is taken from the odds less one, and keeps them all. And more than 709 scales below the centre,
R's `plogis` gives zero where the probability is still a number a double can hold; `cdf` gives
that number.

## A common mistake
Giving a mean and a standard deviation, as `dist.normal` takes:

> dist.logistic(mean: 0, sd: 1)
error[Z0404]: `dist.logistic` has no argument called `mean`
  = help: usage: `dist.logistic(location: Float = 0, scale: Float = 1)`

The centre is `location:`, and a standard deviation of σ is a scale of σ √3 / π:
`dist.logistic(location: 0, scale: 0.5513)` has a standard deviation of one.

## Coming from R or Stata
| dist.logistic(location: m, scale: s).pdf(x) | dlogis(x, m, s) | logisticden(m, s, x) |
| dist.logistic(location: m, scale: s).cdf(x) | plogis(x, m, s) | logistic(m, s, x) |
| dist.logistic(location: m, scale: s).cdf(x, upper: true) | plogis(x, m, s, lower.tail = FALSE) | logistictail(m, s, x) |
| dist.logistic(location: m, scale: s).quantile(p) | qlogis(p, m, s) | invlogistic(m, s, p) |
| dist.logistic(location: m, scale: s).draw(n) | rlogis(n, m, s) | rlogistic(m, s) |

R has four functions for each family, `d`, `p`, `q` and `r` before a short name; here the distribution is one value and the four are its methods. Stata's functions take the parameters first and the value last. R users often call `plogis(x)` and `qlogis(p)` for the inverse logit and the logit of a number; those are `inv_logit` and `logit` here. Stata's `logistic` is also the name of its command for a logistic regression, which is `glm` here.

## Related tasks
- For the density, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, use `quantile`.
- For random numbers, use `draw`.
- For the log-odds of a probability, or a probability from log-odds, use `logit` or `inv_logit`.
- For a logistic regression, use `glm`.
- For a bell-shaped distribution with lighter tails, use `dist.normal`.

## References
- Johnson, N. L., Kotz, S. and Balakrishnan, N. (1995). *Continuous Univariate Distributions*,
  volume 2, 2nd edition, chapter 23. Wiley.
- Balakrishnan, N., editor (1992). *Handbook of the Logistic Distribution*. Marcel Dekker.

## See also
dist.normal, dist.t, logit, inv_logit, glm

## Examples
// The standard logistic: centre 0 and scale 1.
> let d = dist.logistic()
> d
logistic distribution · location: 0, scale: 1

// The density at the centre, the probability below 2 and the upper quartile.
> d.pdf(0)
> d.cdf(2)
> d.quantile(0.75)
0.25
0.880797
1.09861

// A centre of 100 and a scale of 15: the probability above 130.
> dist.logistic(location: 100, scale: 15).cdf(130, upper: true)
0.119203

// A tail far out, computed directly.
> d.cdf(-40)
4.24835e-18
