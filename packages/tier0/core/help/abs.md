# abs

## Summary
Absolute value

## Description
The absolute value of a number: `x` itself when it is not negative, `-x` otherwise. An Int stays an Int
and a Float stays a Float, so `abs(-3)` is `3` and `abs(-2.5)` is `2.5`; `NA` gives `NA`. Given a
column, `abs` applies row by row and returns a column.

The one Int that has no absolute value in 64 bits, -9223372036854775808, is an error (Z0211) rather
than a wrong number.

## Arguments
- x: An Int or a Float, or a column of either.

## Value
An Int for an Int, a Float for a Float; a column of the same when `x` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact: the sign is dropped and nothing is rounded.

## A common mistake
Giving a number that was read as text. A column of numbers with a stray word in it is read as text, and `abs` takes numbers only; the checker refuses the call before anything runs:

> abs("-3")
error[Z0401]: `x` of `abs` must be a Float, but this is a Str
  = help: usage: `abs(x)`

Convert the text first: `abs(to("-3", Float))` is `3`.

## Coming from R or Stata
| abs(x) | abs(x) | abs(x) |

The name and the result are the same in all three, and a missing value stays missing. R and Stata
give the distance without a word in every case; Zeff refuses the one Int whose absolute value does
not fit in 64 bits (Z0211) rather than return a wrong number.

## Related tasks
- For the sign alone, use `sign`.
- For the larger of a value and a floor, as in `max(x, 0)`, use `max`.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.5.1.

## See also
sign, sqrt, max

## Examples
// An Int stays an Int.
> abs(-3)
3

// A Float stays a Float.
> abs(-2.5)
2.5

// Row by row on the shipped wages: how far each of one worker's wages is from 25; the missing wage stays missing.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(gap = abs(wage - 25))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper     gap
  1       19   2012   12.64    29.2       0   12.36
  2       19   2013   25.18    31.4       0    0.18
  3       19   2014   35.49    33.5       1   10.49
  4       19   2015   26.07    58.2       2    1.07
  5       19   2016      NA      30       3      NA
