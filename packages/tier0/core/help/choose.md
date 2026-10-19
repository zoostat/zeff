# choose

## Summary
Number of ways to choose k of n, as a Float

## Description
The binomial coefficient, the number of ways to choose `k` items of `n` without regard to order,
n! / (k! (n − k)!), as a Float. `k` below 0 or above `n` gives 0, as R does; a negative `n` gives
`NA` with a warning (W0205); a result beyond the largest Float gives `Inf` with a warning (W0204).
`NA` gives `NA`. Given columns, `choose` applies row by row and counts the rows that warned in one
line. Like R's `choose()` and Stata's `comb()`.

## Arguments
- n: A whole number from 0, or a column of Ints.
- k: A whole number, or a column of Ints.

## Value
A Float; a column of Floats when either argument is a column.

## Details
The coefficient is computed as a product of exact integers, C(n − k + i, i) at each step, in
128-bit arithmetic while it fits, so every value below 2^53 is exact; beyond that each step
rounds once, and the result is within a few units in the last place of the exact value on every
target. Once `k` exceeds 600 (so `n` exceeds 1,200) the result exceeds the largest Float whatever
`n` is, and is `Inf` at once.

## References
Abramowitz, M. and Stegun, I. A. (1964). *Handbook of Mathematical Functions*, section 24.1.1.
National Bureau of Standards.

## See also
factorial, gamma, log_gamma

## Examples
> choose(5, 2)
10
> choose(50, 25)
126410606437752
> choose(5, 7)
0
> choose(1100, 550)
Inf
warning: choose beyond the largest Float gave Inf (line 4)
  = help: `log_gamma(n + 1) - log_gamma(k + 1) - log_gamma(n - k + 1)` gives the logarithm, which stays finite
