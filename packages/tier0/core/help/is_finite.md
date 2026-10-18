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

## Details
Exact.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 5.7.2 (isFinite).

## See also
is_missing, abs

## Examples
> is_finite(2.5)
true
> is_finite(NA)
false
> is_finite(1 / 0)
false
warning: division by zero gave Inf (line 3)
  = help: test the divisor first, as in `b != 0 && a / b > 1`; on a frame, `where(b != 0)`
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> is_finite(wages["hours"])
Column<Bool> · 3 values
true
true
false
