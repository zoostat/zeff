# sqrt

## Summary
Square root

## Description
The square root of a number, as a Float. A negative number has no real square root: the result is
`NA` with a warning (W0205), as section 4.1 of the Language Reference has it, never a complex number
or a silent `NaN`. `NA` gives `NA`. Given a column, `sqrt` applies row by row, returns a column, and
counts the rows that gave `NA` in one warning.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Correctly rounded: the square root is the one operation beyond the four of arithmetic that IEEE 754
requires to be exact to the last bit, so every target gives the same result. `sqrt(0)` is `0` and
`sqrt(Inf)` is `Inf`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.4.1 (squareRoot).

## See also
abs, exp, log

## Examples
> sqrt(16)
4
> sqrt(2)
1.41421
> sqrt(-1)
NA
warning: sqrt of a negative number gave NA (line 3)
  = help: a negative number has no real square root; `where(x >= 0)` keeps the rows that have one
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> sqrt(wages["hours"])
Column<Float> · 3 values
6.32456
5.91608
NA
