# beta

## Summary
Beta function (the distribution is dist.beta)

## Description
B(a, b) = Γ(a) Γ(b) / Γ(a + b), the beta function, for positive `a` and `b`; it is the
normalising constant of the beta distribution and of the binomial coefficient's continuous
extension, 1 / (n + 1) C(n, k) = B(k + 1, n − k + 1). An argument that is not positive gives
`NA` with a warning (W0205); a result beyond the largest Float gives `Inf` with a warning (W0204).
`NA` gives `NA`. Given columns, `beta` applies row by row and counts the rows that warned in one
line. Like R's `beta()`; Stata has `lnbeta()` for its logarithm. The distribution of that name is
`dist.beta`.

## Arguments
- a: A positive number, or a column.
- b: A positive number, or a column.

## Value
A Float; a column of Floats when either argument is a column.

## Missing values
`NA` in either argument gives `NA`. An argument that is not positive also gives `NA`, with warning W0205; in columns the rows that did so are counted in one warning.

## Details
Computed as the exponential of ln B(a, b), which is formed from `log_gamma` of the smaller argument
and the Stirling series of the larger, so that large arguments lose no digits to cancellation;
the value is within 5e-13 relative of the exact one and the same on every target. B(a, b) = B(b, a),
and B(1, 1) = 1.

## A common mistake
Giving zero or a negative number. The function is defined for positive arguments, so the result is `NA`, and the warning says so:

> beta(0, 1)
NA
warning[W0205]: beta with an argument that is not positive gave NA (line 5)
  = help: B(a, b) is defined for positive a and b

Both arguments must be above zero: `beta(0.5, 1)` is `2`.

## Coming from R or Stata
| beta(a, b) | beta(a, b) | exp(lngamma(a) + lngamma(b) - lngamma(a + b)) |

R's function is the same. The beta distribution is `dist.beta` here, `dbeta` and its family in R
and `betaden` and its family in Stata.

## Related tasks
- For the beta distribution, use `dist.beta`.

## References
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 6.2.
  National Bureau of Standards.
- Olver, F. W. J. and others (editors). *NIST Digital Library of Mathematical Functions*,
  section 5.12, https://dlmf.nist.gov/5.12.

## See also
gamma, log_gamma, choose, dist.beta

## Examples
// A small case: 1 / 12.
> beta(2, 3)
0.0833333

// At one half and one half, pi.
> beta(0.5, 0.5)
3.14159

// The order of the arguments does not matter.
> beta(3, 2)
0.0833333

// Row by row.
> frame(a = [0.5, 1, 2, NA], b = [0.5, 1, 3, 2]).add(value = beta(a, b))
frame · 4 rows × 3 variables

        a     b       value
  1   0.5   0.5     3.14159
  2     1     1           1
  3     2     3   0.0833333
  4    NA     2          NA
