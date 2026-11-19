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

## Missing values
`NA` gives `NA`. A negative number also gives `NA`, with warning W0205, and a number above 170 gives `Inf`, with warning W0204; in a column the rows of each are counted in one warning.

## Details
Each value from 0! to 170! is the double nearest the exact integer, held in a table, so every
factorial is correctly rounded on every target. For other arguments see `gamma`: Γ(n + 1) = n!,
and `log_gamma(n + 1)` gives log n! without overflow.

## A common mistake
Asking for a factorial that no Float holds. Above 170 the result is `Inf`, and the warning names the way out:

> factorial(171)
Inf
warning[W0204]: factorial above 170 gave Inf (line 5)
  = help: 171! exceeds the largest Float; `log_gamma(n + 1)` gives the logarithm, which stays finite

Work with the logarithm, which stays finite: `log_gamma(171 + 1)` is the logarithm of 171!.

## Coming from R or Stata
| factorial(n) | factorial(n) | exp(lnfactorial(n)) |

Stata has the logarithm only. R returns `NaN` with a warning for a negative number; Zeff returns
`NA` and counts the rows.

## Related tasks
- For the number of ways to choose some of them, use `choose`.
- For the logarithm of a large factorial, use `log_gamma`.

## References
Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 6.1.
National Bureau of Standards.

## See also
choose, gamma, log_gamma

## Examples
// Five items can be put in 120 orders.
> factorial(5)
120

// Beyond what an Int holds, as a Float.
> factorial(25)
1.55112e+25

// The largest that is finite.
> factorial(170)
7.25742e+306

// Row by row; nothing to order is one order.
> frame(n = [0, 1, 5, 10, NA]).add(orders = factorial(n))
frame · 5 rows × 2 variables

       n    orders
  1    0         1
  2    1         1
  3    5       120
  4   10   3628800
  5   NA        NA
