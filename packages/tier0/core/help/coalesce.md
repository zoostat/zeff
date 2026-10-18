# coalesce

## Summary
First non-missing value

## Description
The first of its arguments that is not `NA`, or `NA` when all of them are. The arguments must be of
one type, Int and Float joining as Float (section 3.7 of the Language Reference, "Joining types"):
`coalesce(NA, 2)` is `2` and `coalesce(NA, NA, "none")` is `none`, while `coalesce(1, "a")` is
refused (Z0214). Given columns, `coalesce` applies row by row, a scalar standing for its value in
every row, and returns a column: `coalesce(wages["hours"], 0)` fills the gaps in hours with zero, and
`coalesce(a, b)` takes `b` wherever `a` is missing, as dplyr's `coalesce` does.

## Arguments
- a: A value of a computable type (Bool, Int, Float or Str), or a column of one.
- b: Another of the same type.
- ...: Further values or columns of the same type.

## Value
A value of the arguments' joined type; a column of it when any argument is a column.

## Details
Exact. Columns must have the same number of rows (Z0217).

## References
Language Reference, sections 3.7 ("Joining types") and 4.1.

## See also
is_missing, min, max

## Examples
> coalesce(NA, 2)
2
> coalesce(NA, NA, "none")
none
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> coalesce(wages["hours"], 0)
Column<Int> · 3 values
40
35
0
> coalesce(wages["hours"], wages["wage"])
Column<Float> · 3 values
40
35
31.2
