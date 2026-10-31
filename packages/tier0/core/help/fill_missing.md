# fill_missing

## Summary
Replace NA with a value

## Description
`x` where it has a value and `value` where it is `NA`: `coalesce(x, value)` under another name,
for the common case of one fallback. The two must be of one type under the joining rule of
section 3.7 of the Language Reference: an Int fills a list or column of Floats as a Float, and a
text cannot fill numbers (Z0214). Given a column, it applies row by row and gives a column with
no `NA`, so `n_missing(fill_missing(x, 0))` is 0. Filling is a modelling decision, not a
cleaning step: a zero in place of a missing wage changes every mean; `drop_missing` (release
0.5.0, `data` package) and the reducers' own skipping are the alternatives. Like tidyr's
`replace_na` and R's `replace(x, is.na(x), value)`.

## Arguments
- x: A value or a column, possibly `NA`.
- value: What stands in for `NA`, of the same type.

## Value
The joined type of `x` and `value`; a column when `x` is one.

## See also
coalesce, n_missing, is_missing

## Examples
> fill_missing(to(NA, Int), 0)
0
> fill_missing(2.5, 0)
2.5
> let d = frame(x = [1, NA, 3])
> fill_missing(d["x"], 0)
Column<Int> · 3 values
1
0
3
> n_missing(fill_missing(d["x"], 0))
0
