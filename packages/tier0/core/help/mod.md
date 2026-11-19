# mod

## Summary
Remainder after floored division

## Description
The remainder of `a` after floored division by `b`, with the sign of `b`, as Python's `%`, R's `%%`
and Stata's `mod` compute it: `mod(7, 2)` is `1`, `mod(-7, 2)` is `1` and `mod(7, -2)` is `-1`, so
that `a == b * div(a, b) + mod(a, b)` always holds. With two Ints the result is an Int, and a divisor
of zero is an error (Z0212); with a Float the result is a Float, and a divisor of zero or an infinite
dividend gives `NA` with a warning. `NA` gives `NA`. Given columns, `mod` applies row by row and
returns a column.

## Arguments
- a: The dividend, an Int or a Float, or a column of either.
- b: The divisor, likewise.

## Value
An Int when both are Ints, a Float otherwise; a column of the same when an argument is a column.

## Missing values
`NA` in either argument gives `NA`. In columns each row is taken by itself, and no row is dropped.

## Details
With Floats the remainder is the exact IEEE remainder moved into `[0, b)` or `(b, 0]`, so
`mod(7.5, 0.1)` is `0.0999999999999996` (printed `0.1`): 0.1 as the machine holds it is a little
more than a tenth, and 74 of them fit in 7.5. `mod(a, Inf)` is `a` when the signs agree.

## A common mistake
Taking the remainder of whole numbers by zero. There is none, and the statement stops:

> mod(7, 0)
error[Z0212]: division by zero: `mod(7, 0)` has no value
  = help: test the divisor first, as in `b != 0 && div(a, b) > 1`; with Floats, `div` and `mod` follow `/` and give Inf or NA with a warning

Keep the rows with a divisor first, `where(b != 0)`; with a Float, `mod(7.0, 0)` gives `NA` with a warning.

## Coming from R or Stata
| mod(a, b) | a %% b | mod(a, b) |

The three give the remainder the sign of the divisor. For a divisor of zero R's `%%` on integers
gives `NA` and Stata's `mod` gives missing; Zeff stops with an error for Ints.

## Related tasks
- For the quotient, use `div`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.3.1 (remainder).

## See also
div, floor

## Examples
// The remainder.
> mod(7, 2)
1

// It has the sign of the divisor.
> mod(-7, 2)
1

// And of a negative divisor.
> mod(7, -2)
-1

// Row by row on the shipped wages: which years are even.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(even = mod(year, 2) == 0)
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   even
  1       19   2012   12.64    29.2       0   true
  2       19   2013   25.18    31.4       0   false
  3       19   2014   35.49    33.5       1   true
  4       19   2015   26.07    58.2       2   false
  5       19   2016      NA      30       3   true
