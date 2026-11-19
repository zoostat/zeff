# is_finite

## Summary
True unless x is NA, Inf or -Inf

## Description
Whether a number is finite: `true` for every Int and every ordinary Float, `false` for `Inf`, `-Inf`
and `NA`, as R's `is.finite`. It is the test to run after a division that may have produced an
infinity, or before a logarithm or a square root. Given a column, `is_finite` applies row by row and
returns a column of Bools with no `NA` in it.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Bool; a column of Bools when `x` is a column.

## Missing values
`NA` gives `false`: a missing number is not a finite one. The result therefore has no `NA` in it, and nothing is printed.

## Details
Exact.

## Coming from R or Stata
| is_finite(x) | is.finite(x) | !missing(x) |

R agrees. Stata has no infinity: a number too large to hold is missing there, so the test for a
usable number is `!missing(x)`.

## Related tasks
- To test for `NA` alone, use `is_missing`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.7.2 (isFinite).

## See also
is_missing, abs

## Examples
// An ordinary number.
> is_finite(2.5)
true

// A missing number is not finite.
> is_finite(NA)
false

// Nor is an infinity.
> is_finite(-Inf)
false

// Row by row on the shipped wages: the logarithm of experience is `-Inf` in the two years of none, and the test finds them.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(lexper = log(exper), usable = is_finite(lexper))
frame · 5 rows × 7 variables · panel worker × year

      worker   year    wage   hours   exper     lexper   usable
  1       19   2012   12.64    29.2       0       -Inf   false
  2       19   2013   25.18    31.4       0       -Inf   false
  3       19   2014   35.49    33.5       1          0   true
  4       19   2015   26.07    58.2       2   0.693147   true
  5       19   2016      NA      30       3    1.09861   true
warning[W0204]: log of zero gave -Inf in 2 rows (line 5)
  = help: `log1p(x)` is log(1 + x) and is finite at zero; or keep the rows with `where(x > 0)`
