# round

## Summary
Round to decimal places or significant digits

## Description
`x` rounded to `digits:` decimal places (none by default; a negative count rounds to tens, hundreds
and so on) or, with `sig:`, to that many significant digits, as a Float. A value exactly halfway
between two candidates goes to the even one, as R rounds (`round(2.5)` is `2`, `round(3.5)` is
`4`); `mode: away` sends it away from zero instead, as Stata and Excel round (`round(2.5, mode:
away)` is `3`). `NA` gives `NA`. Given a column, `round` applies row by row and returns a column of
Floats.

Whether a value is halfway is decided on the number as the machine holds it: `2.675` is a little
below 2.675 in binary, so `round(2.675, digits: 2)` is `2.67` under both modes, as in Python and
Stata; R 4 rounds a few such numbers the other way.

## Arguments
- x: An Int or a Float, or a column of either.
- digits: The number of decimal places, an Int; 0 by default. Negative counts round to the left of
  the decimal point: `digits: -2` rounds to hundreds.
- sig: The number of significant digits, an Int of at least 1, instead of `digits:`; the two
  cannot be given together (Z0410).
- mode: `even` (the default) rounds a tie to the even candidate; `away` rounds it away from zero.

## Value
A Float; a column of Floats when `x` is a column.

## Details
The tie test is exact: a Float is halfway when, written as an odd whole number times a power of two,
the power is exactly one below the decimal place asked for. The result is the Float nearest to the
decimal chosen, so `round(0.125, digits: 2)` is the Float nearest 0.12 and prints as `0.12`.
The same tie test serves `sig:`, which rounds to `sig - 1 - floor(log10(|x|))` decimal places.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 4.3 (rounding-direction
attributes).

## See also
floor, ceil, truncate

## Examples
> round(2.5)
2
> round(3.5)
4
> round(2.5, mode: away)
3
> round(2.675, digits: 2)
2.67
> round(1250, digits: -2)
1200
> round(123456.789, sig: 3)
123000
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 7)
> round(wages["wage"])
Column<Float> · 3 values
23
19
31
