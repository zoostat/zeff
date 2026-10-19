# factorial

## Summary
n! as a Float, so factorial(25) is finite

## Description
The factorial of a whole number, n × (n − 1) × … × 1, as a Float, so that it does not stop at the
20! that fits a 64-bit Int: `factorial(25)` is 1.55112e+25 and `factorial(170)` is the largest
finite one. A negative `n` gives `NA` with a warning (W0205), as section 4.1 of the Language
Reference has it; `n` above 170 gives `Inf` with a warning (W0204). `NA` gives `NA`. Given a
column, `factorial` applies row by row, returns a column, and counts the rows that warned in one
line. Like R's `factorial()`; Stata has no factorial function and uses `exp(lnfactorial())`.

## Arguments
- n: A whole number from 0, or a column of Ints.

## Value
A Float; a column of Floats when `n` is a column.

## Details
Each value from 0! to 170! is the double nearest the exact integer, held in a table, so every
factorial is correctly rounded on every target. For other arguments see `gamma`: Γ(n + 1) = n!,
and `log_gamma(n + 1)` gives log n! without overflow.

## References
Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 6.1.
National Bureau of Standards.

## See also
choose, gamma, log_gamma

## Examples
> factorial(5)
120
> factorial(20)
2.4329e+18
> factorial(170)
7.25742e+306
> factorial(171)
Inf
warning: factorial above 170 gave Inf (line 4)
  = help: 171! exceeds the largest Float; `log_gamma(n + 1)` gives the logarithm, which stays finite
> factorial(-3)
NA
warning: factorial of a negative number gave NA (line 5)
  = help: n! is defined for whole numbers from 0; `gamma(x + 1)` extends it to other numbers
