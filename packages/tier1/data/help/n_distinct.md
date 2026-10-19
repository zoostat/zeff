# n_distinct

## Summary
Number of distinct values

## Description
The number of distinct values of a variable, by identity (section 3.7 of the Language Reference):
`NA` counts as one value, and `-0.0` is `0`. As a reducer inside `add` or `update`, it counts within
the frame or, under `by:`, within the group; on a column written `wages["region"]` it counts the
whole column. Like dplyr's `n_distinct`.

## Arguments
- x: A column, named bare inside a frame method or as `frame["x"]`.

## Value
An `Int`.

## Details
Exact: the distinct keys under the identity relation of section 3.7, which `sort` and grouping use too.

## References
Language Reference, section 3.7 ("Identity").

## See also
n, summary

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> n_distinct(wages["hours"])
3
> wages.add(kinds = n_distinct(hours))["kinds"]
Column<Int> · 3 values
3
3
3
