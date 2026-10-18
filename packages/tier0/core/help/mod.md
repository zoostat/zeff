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

## Details
With Floats the remainder is the exact IEEE remainder moved into `[0, b)` or `(b, 0]`, so
`mod(7.5, 0.1)` is `0.0999999999999996` (printed `0.1`): 0.1 as the machine holds it is a little
more than a tenth, and 74 of them fit in 7.5. `mod(a, Inf)` is `a` when the signs agree.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.3.1 (remainder).

## See also
div, floor

## Examples
> mod(7, 2)
1
> mod(-7, 2)
1
> mod(7, -2)
-1
> mod(7.5, 2)
1.5
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 5)
> mod(wages["hours"], 7)
Column<Int> · 3 values
5
0
NA
