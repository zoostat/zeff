# is_missing

## Summary
True where x is NA

## Description
Whether a value is missing: `true` for `NA` and `false` for anything else, including `Inf`, zero and
the empty text. It is the test to write where `x == NA` would not do: a comparison with `NA` is
always `NA` (section 4.1 of the Language Reference), so it never selects anything. Given a column,
`is_missing` applies row by row and returns a column of Bools with no `NA` in it, which is what
`where` needs to keep or drop the rows with gaps. R has `is.na`; Stata has `missing`.

## Arguments
- x: A value of any computable type (Bool, Int, Float or Str), or a column of one.

## Value
A Bool; a column of Bools when `x` is a column.

## Details
Exact.

## References
Language Reference, section 4.1 (the rules of `NA`).

## See also
coalesce, is_finite

## Examples
> is_missing(NA)
true
> is_missing(0)
false
> is_missing("")
false
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> is_missing(wages["hours"])
Column<Bool> · 3 values
false
false
true
