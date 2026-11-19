# div

## Summary
Floored division

## Description
The quotient of `a` by `b` rounded towards negative infinity, as Python's `//` and R's `%/%`
compute it: `div(7, 2)` is `3` and `div(-7, 2)` is `-4`, so that `a == b * div(a, b) + mod(a, b)`
always holds. With two Ints the result is an Int, and a divisor of zero is an error (Z0212); with a
Float the result is a Float and follows `/`: a division by zero gives `Inf` or `-Inf` with a warning
and `0 / 0` gives `NA` with a warning. `NA` gives `NA`. Given columns, `div` applies row by row and
returns a column.

The operator `/` always gives a Float; `div` is the way to a whole-number quotient.

## Arguments
- a: The dividend, an Int or a Float, or a column of either.
- b: The divisor, likewise.

## Value
An Int when both are Ints, a Float otherwise; a column of the same when an argument is a column.

## Missing values
`NA` in either argument gives `NA`. In columns each row is taken by itself, and no row is dropped.

## Details
With Floats the quotient is taken from the exact remainder, never from `floor(a / b)`, which would
round `7.5 / 0.1` up to 75 although 0.1 is a little more than a tenth as the machine holds it:
`div(7.5, 0.1)` is `74`, as in Python and R. The one Int quotient that does not fit in 64 bits,
`div(-9223372036854775808, -1)`, is an error (Z0211).

## A common mistake
Dividing whole numbers by zero. There is no whole-number quotient, and the statement stops:

> div(7, 0)
error[Z0212]: division by zero: `div(7, 0)` has no value
  = help: test the divisor first, as in `b != 0 && div(a, b) > 1`; with Floats, `div` and `mod` follow `/` and give Inf or NA with a warning

Keep the rows with a divisor first, `where(b != 0)`; with a Float, `div(7.0, 0)` follows `/` and gives `Inf` with a warning.

## Coming from R or Stata
| div(a, b) | a %/% b | floor(a / b) |

Stata has no operator for it, and `int(a / b)` there rounds towards zero, so it differs for a
negative quotient: `div(-7, 2)` is `-4`, `int(-7 / 2)` is `-3`. R's `%/%` on integers gives `NA`
for a divisor of zero; Zeff stops with an error.

## Related tasks
- For the remainder, use `mod`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.3.1 (remainder).

## See also
mod, floor

## Examples
// A whole-number quotient.
> div(7, 2)
3

// Rounded down, not towards zero.
> div(-7, 2)
-4

// With a Float the result is a Float.
> div(7.5, 2)
3

// Row by row on the shipped wages: the decade of each year.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(decade = div(year, 10) * 10)
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   decade
  1       19   2012   12.64    29.2       0     2010
  2       19   2013   25.18    31.4       0     2010
  3       19   2014   35.49    33.5       1     2010
  4       19   2015   26.07    58.2       2     2010
  5       19   2016      NA      30       3     2010
