# sign

## Summary
-1, 0 or 1

## Description
The sign of a number as an Int: `-1` for a negative number, `0` for zero and `1` for a positive one;
`NA` gives `NA`. Given a column, `sign` applies row by row and returns a column of Ints.

R's `sign` returns a double; Zeff's returns an Int, since the three values are whole numbers.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
An Int; a column of Ints when `x` is a column.

## Details
Exact. Both zeros (`0` and `-0.0`) give `0`; `Inf` gives `1` and `-Inf` gives `-1`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.7.2 (sign of a value).

## See also
abs, is_finite

## Examples
> sign(-2.5)
-1
> sign(0)
0
> sign(7)
1
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> sign(wages["wage"] - 25)
Column<Int> · 3 values
-1
-1
1
