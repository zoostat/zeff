# n_rows

## Summary
Number of rows

## Description
The number of rows of a frame, as an `Int`: every row, whether or not it holds missing values. A
frame read from a file has one row per data line of the file; a frame built with `frame` has as
many rows as each of its lists has values. Of a printed table, such as the one `duplicates`
gives, it is the number of rows of the table: `d.duplicates(firm).n_rows()` is the number of
repeated values.

Like R's `nrow` and Stata's `count` without a condition.

## Arguments
- data: The frame, or a table.

## Value
An `Int`, `0` for a frame with no rows.

## See also
n_cols, variables, describe

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.n_rows()
3
> frame(x = [1, 2]).n_rows()
2
