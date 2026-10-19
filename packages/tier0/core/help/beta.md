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

## Details
Computed as the exponential of ln B(a, b), which is formed from `log_gamma` of the smaller argument
and the Stirling series of the larger, so that large arguments lose no digits to cancellation;
the value is within 5e-13 relative of the exact one and the same on every target. B(a, b) = B(b, a),
and B(1, 1) = 1.

## References
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 6.2.
  National Bureau of Standards.
- Olver, F. W. J. and others (editors). *NIST Digital Library of Mathematical Functions*,
  section 5.12, https://dlmf.nist.gov/5.12.

## See also
gamma, log_gamma, choose, dist.beta

## Examples
> beta(2, 3)
0.0833333
> beta(0.5, 0.5)
3.14159
> beta(0, 1)
NA
warning: beta with an argument that is not positive gave NA (line 3)
  = help: B(a, b) is defined for positive a and b
