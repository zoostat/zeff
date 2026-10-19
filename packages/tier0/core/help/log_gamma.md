# log_gamma

## Summary
Logarithm of the gamma function

## Description
ln |Γ(x)|, the natural logarithm of the absolute value of the gamma function, which stays finite
where Γ itself overflows: `log_gamma(1000)` is 5905.22 while `gamma(1000)` is beyond any Float.
At the poles of Γ, zero and the negative whole numbers, the result is `Inf` with a warning
(W0204). `NA` gives `NA`. Given a column, `log_gamma` applies row by row, returns a column, and
counts the rows that warned in one line. Like R's `lgamma()` and Stata's `lngamma()`.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
The `lgamma` of the engine's one pure-Rust maths library (a port of musl's), within a few units
in the last place of the exact value and exactly 0 at 1 and 2, the same bits on every target. It
is the function the distribution functions build on: the normalising constants of the gamma, beta,
t, F and chi-square distributions are differences of `log_gamma` values, formed so that large
arguments lose no digits to cancellation.

## References
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 6.1.
  National Bureau of Standards.
- Olver, F. W. J. and others (editors). *NIST Digital Library of Mathematical Functions*,
  section 5.11, https://dlmf.nist.gov/5.11.

## See also
gamma, factorial, beta

## Examples
> log_gamma(100)
359.134
> log_gamma(0.5)
0.572365
> log_gamma(1000)
5905.22
> log_gamma(-1)
Inf
warning: log_gamma at a pole gave Inf (line 4)
  = help: Γ has poles at zero and the negative whole numbers, so its logarithm is infinite there
