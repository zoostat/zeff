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

## Details
At the whole numbers 1 to 171 the value is the exact factorial from the table `factorial` uses;
elsewhere it is the `tgamma` of the engine's one pure-Rust maths library (a port of musl's),
within two units in the last place of the exact value, the same bits on every target. The
reflection formula Γ(x) Γ(1 − x) = π / sin(πx) serves negative arguments.

## References
- Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, chapter 6.
  National Bureau of Standards.
- Olver, F. W. J. and others (editors). *NIST Digital Library of Mathematical Functions*,
  chapter 5, https://dlmf.nist.gov/5.

## See also
log_gamma, factorial, beta, dist.gamma

## Examples
> gamma(0.5)
1.77245
> gamma(5)
24
> gamma(-1.5)
2.36327
> gamma(0)
NA
warning: gamma at a pole gave NA (line 4)
  = help: Γ has poles at zero and the negative whole numbers; between them it is finite
> gamma(200)
Inf
warning: gamma above 171.6 gave Inf (line 5)
  = help: Γ(x) exceeds the largest Float; `log_gamma(x)` gives the logarithm, which stays finite
