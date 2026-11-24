# quantile

## Summary
Value with cumulative probability p

## In brief
- `d.quantile(p)`: the value with probability `p` at or below it.
- `d.quantile(column)`: the same for each row of a column of probabilities.

## Description
The quantile of a distribution at probability `p`: the smallest x with P(X ≤ x) ≥ p, which for a
continuous distribution is the x with P(X ≤ x) = p exactly. `quantile(0.975)` of the standard
normal is the familiar 1.96. Given a column, `quantile` applies row by row and returns a column.
`NA` gives `NA`; a `p` outside [0, 1] is refused (Z0413). The sample quantile of a variable is the frame method of the same name,
`wages.quantile(wage, p: 0.9)`.

## Arguments
- distribution: The distribution, a `Distribution` value such as `dist.normal()`.
- p: A probability between 0 and 1, or a column of them.

## Value
A Float; a column of Floats when `p` is a column. At `p` = 0 or 1 the end of the support, `-Inf`
or `Inf` for an unbounded one.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, and nothing is printed for it.

## Details
Each family inverts its own distribution function: in closed form where one exists (normal
through the inverse of the error function, exponential, uniform, log-normal) and by the inverse
incomplete beta and gamma functions of the kernel for the t, F, chi-square, gamma and beta
families, to full double precision on every target.

## A common mistake
Writing the probability as a percentage:

> dist.normal().quantile(97.5)
error[Z0413]: `p` of `quantile` must be a probability between 0 and 1, but this is 97.5

`p` is a probability between 0 and 1: `dist.normal().quantile(0.975)`.

## Coming from R or Stata
| dist.normal().quantile(p) | qnorm(p) | invnormal(p) |
| dist.t(df: v).quantile(p) | qt(p, v) | invt(v, p) |
| dist.chi2(df: v).quantile(p) | qchisq(p, v) | invchi2(v, p) |

The probability is the one below the value, as in R. Stata also has `invttail`, `invchi2tail`
and `invFtail`, which take the probability above it; the upper critical value at five per cent
is `quantile(0.95)` here.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For random numbers, use `draw`.
- For the quantile of a variable of data, use the frame method of `data` that is also called `quantile`.

## References
- The help page of each family (`zeff help dist.normal`, …) for its formula and reference.

## See also
pdf, cdf, draw

## Examples
// The two-sided critical value of the standard normal at five per cent.
> dist.normal().quantile(0.975)
1.95996

// Of any distribution: the ninth decile of a score, the median of a waiting time, a quartile of a uniform.
> dist.normal(mean: 100, sd: 15).quantile(0.9)
> dist.exponential(rate: 2).quantile(0.5)
> dist.uniform(min: 2, max: 5).quantile(0.25)
119.223
0.346574
2.75

// For a count, the smallest count with at least that probability at or below it.
> dist.poisson(lambda: 2).quantile(0.9)
4

// It undoes `cdf`.
> dist.t(df: 24).cdf(dist.t(df: 24).quantile(0.975))
0.975

// Row by row on a column of probabilities.
> frame(p = [0.025, 0.5, 0.975]).add(z = dist.normal().quantile(p))
frame · 3 rows × 2 variables

          p          z
  1   0.025   -1.95996
  2     0.5          0
  3   0.975    1.95996
