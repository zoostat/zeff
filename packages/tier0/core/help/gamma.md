# gamma

## Summary
Gamma function (the distribution is dist.gamma)

## Description
Γ(x), the gamma function, which extends the factorial to every real number: Γ(n) = (n − 1)! at
the whole numbers, and Γ(½) = √π. It has poles at zero and the negative whole numbers, where the
result is `NA` with a warning (W0205), as section 4.1 of the Language Reference has it; above
171.6 it exceeds the largest Float and gives `Inf` with a warning (W0204). `NA` gives `NA`. Given a
column, `gamma` applies row by row, returns a column, and counts the rows that warned in one line.
Like R's `gamma()` and Stata's `exp(lngamma())`; the distribution of that name is `dist.gamma`.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Missing values
`NA` gives `NA`. A pole also gives `NA`, with warning W0205, and a number above 171.6 gives `Inf`, with warning W0204; in a column the rows of each are counted in one warning.

## Details
At the whole numbers 1 to 171 the value is the exact factorial from the table `factorial` uses;
elsewhere it is the `tgamma` of the engine's one pure-Rust maths library (a port of musl's),
within two units in the last place of the exact value, the same bits on every target. The
reflection formula Γ(x) Γ(1 − x) = π / sin(πx) serves negative arguments.

## A common mistake
Asking for a value that no Float holds. Above 171.6 the result is `Inf`, and the warning names the way out:

> gamma(200)
Inf
warning[W0204]: gamma above 171.6 gave Inf (line 5)
  = help: Γ(x) exceeds the largest Float; `log_gamma(x)` gives the logarithm, which stays finite

Work with the logarithm, which stays finite: `log_gamma(200)`.

## Coming from R or Stata
| gamma(x) | gamma(x) | exp(lngamma(x)) |

Stata has the logarithm only. At a pole R returns `NaN` with a warning; Zeff returns `NA` and says
that it is a pole. The gamma distribution is `dist.gamma` here, `dgamma` and its family in R.

## Related tasks
- For the logarithm, which does not overflow, use `log_gamma`.
- For the gamma distribution, use `dist.gamma`.

## References
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, chapter 6.
  National Bureau of Standards.
- Olver, F. W. J. and others (editors). *NIST Digital Library of Mathematical Functions*,
  chapter 5, https://dlmf.nist.gov/5.

## See also
log_gamma, factorial, beta, dist.gamma

## Examples
// At a whole number, the factorial of the one before.
> gamma(5)
24

// At one half, the square root of pi.
> gamma(0.5)
1.77245

// Between the poles of the negative numbers it is finite.
> gamma(-1.5)
2.36327

// Row by row.
> frame(x = [0.5, 1, 2.5, 5, NA]).add(g = gamma(x))
frame · 5 rows × 2 variables

        x         g
  1   0.5   1.77245
  2     1         1
  3   2.5   1.32934
  4     5        24
  5    NA        NA
