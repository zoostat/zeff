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

## Missing values
`NA` gives `NA`. A pole gives `Inf`, with warning W0204; in a column the rows that did so are counted in one warning.

## Details
The `lgamma` of the engine's one pure-Rust maths library (a port of musl's), within a few units
in the last place of the exact value and exactly 0 at 1 and 2, the same bits on every target. It
is the function the distribution functions build on: the normalising constants of the gamma, beta,
t, F and chi-square distributions are differences of `log_gamma` values, formed so that large
arguments lose no digits to cancellation.

## A common mistake
Giving zero or a negative whole number, where the gamma function has a pole. The result is `Inf`, and the warning says why:

> log_gamma(-1)
Inf
warning[W0204]: log_gamma at a pole gave Inf (line 5)
  = help: Γ has poles at zero and the negative whole numbers, so its logarithm is infinite there

Between the poles the function is finite: `log_gamma(-1.5)` is `0.860047`.

## Coming from R or Stata
| log_gamma(x) | lgamma(x) | lngamma(x) |
| log_gamma(n + 1) | lfactorial(n) | lnfactorial(n) |

The three agree wherever the gamma function is finite; each gives the logarithm of its size, so
the sign of a negative value is lost in all three.

## Related tasks
- For the function itself, where it does not overflow, use `gamma`.

## References
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 6.1.
  National Bureau of Standards.
- Olver, F. W. J. and others (editors). *NIST Digital Library of Mathematical Functions*,
  section 5.11, https://dlmf.nist.gov/5.11.

## See also
gamma, factorial, beta

## Examples
// The logarithm of 99!.
> log_gamma(100)
359.134

// Finite where `gamma` overflows.
> log_gamma(1000)
5905.22

// The logarithm of a large binomial coefficient, 1100 choose 550.
> log_gamma(1101) - 2 * log_gamma(551)
758.734

// Row by row.
> frame(x = [0.5, 1, 10, 100, NA]).add(lg = log_gamma(x))
frame · 5 rows × 2 variables

        x         lg
  1   0.5   0.572365
  2     1          0
  3    10    12.8018
  4   100    359.134
  5    NA         NA
