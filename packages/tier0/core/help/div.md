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

## Details
With Floats the quotient is taken from the exact remainder, never from `floor(a / b)`, which would
round `7.5 / 0.1` up to 75 although 0.1 is a little more than a tenth as the machine holds it:
`div(7.5, 0.1)` is `74`, as in Python and R. The one Int quotient that does not fit in 64 bits,
`div(-9223372036854775808, -1)`, is an error (Z0211).

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.3.1 (remainder).

## See also
mod, floor

## Examples
> div(7, 2)
3
> div(-7, 2)
-4
> div(7.5, 2)
3
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> div(wages["hours"], 7)
Column<Int> · 3 values
5
5
NA
