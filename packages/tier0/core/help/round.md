# round

## Summary
Round to decimal places or significant digits

## In brief
- `round(x)`: to the nearest whole number, a tie going to the even one.
- `round(x, digits: 2)`: to two decimal places.
- `round(x, sig: 3)`: to three significant digits.

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

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
The tie test is exact: a Float is halfway when, written as an odd whole number times a power of two,
the power is exactly one below the decimal place asked for. The result is the Float nearest to the
decimal chosen, so `round(0.125, digits: 2)` is the Float nearest 0.12 and prints as `0.12`.
The same tie test serves `sig:`, which rounds to `sig - 1 - floor(log10(|x|))` decimal places.

## A common mistake
Giving both `digits:` and `sig:`. They are two ways of saying where to round, and only one can hold:

> round(2.567, digits: 1, sig: 2)
error[Z0410]: `round` takes `digits:` or `sig:`, not both
  = help: `digits:` counts decimal places and `sig:` significant digits; keep the one you mean

Give one of them: `round(2.567, digits: 1)` is `2.6`, and `round(2.567, sig: 2)` is `2.6` as well.

## Coming from R or Stata
| round(x, digits: 2) | round(x, 2) | round(x, 0.01) |
| round(x, sig: 3) | signif(x, 3) | no single function |
| round(x, mode: away) | no single function | round(x) |

Stata's second argument is the unit to round to, not a number of digits: `round(x, 0.01)` there is
`round(x, digits: 2)` here. A tie goes to the even number by default, as in R; `mode: away` gives
the rounding of Stata and of a spreadsheet.

## Related tasks
- To drop the fraction without rounding, use `floor`, `ceil` or `truncate`.
- To print with fewer digits and keep the value, use `format`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 4.3 (rounding-direction
attributes).

## See also
floor, ceil, truncate

## Examples
// A tie goes to the even whole number, as in R.
> round(2.5)
2

// With `mode: away` it goes away from zero, as in Stata.
> round(2.5, mode: away)
3

// Decimal places.
> round(3.14159, digits: 2)
3.14

// A negative count rounds to tens, hundreds and so on.
> round(1250, digits: -2)
1200

// Significant digits.
> round(123456.789, sig: 3)
123000

// Row by row on the shipped wages: each wage to the nearest whole number.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(whole = round(wage))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   whole
  1       19   2012   12.64    29.2       0      13
  2       19   2013   25.18    31.4       0      25
  3       19   2014   35.49    33.5       1      35
  4       19   2015   26.07    58.2       2      26
  5       19   2016      NA      30       3      NA
