# n_rows

## Summary
Number of rows

## Description
The number of rows of a frame, as an `Int`: every row, whether or not it holds missing values. A
frame read from a file has one row per data line of the file; a frame built with `frame` has as
many rows as each of its lists has values. Of a printed table, such as the one `duplicates`
gives, it is the number of rows of the table: `d.duplicates(firm).n_rows()` is the number of
repeated values.

## Arguments
- data: The frame, or a table.

## Value
An `Int`, `0` for a frame with no rows.

## Missing values
Every row is counted, whatever it holds: a row whose values are all `NA` is a row.

## Coming from R or Stata
| d.n_rows() | nrow(d) | count, or display _N |
| d.where(wage > 20).n_rows() | sum(d$wage > 20, na.rm = TRUE), or nrow(subset(d, wage > 20)) | count if wage > 20 & !missing(wage) |

Stata's `count if` counts the rows that meet a condition, and a missing value counts as larger
than any number there, so `count if wage > 20` includes the rows with no wage; here `where`
keeps a row only when the condition is true.

## Related tasks
- For the number of variables, use `n_cols`.
- For the number of rows in each group, use `n`.
- For the missing values of a variable, use `n_missing`.
- For the count of each value of a variable, use `tabulate`.

## See also
n_cols, variables, describe

## Examples
// The rows of the shipped wages.
> let wages = dataset(wages)
> wages.n_rows()
18412

// The rows that meet a condition.
> wages.where(year == 2019).n_rows()
1518

// A frame can have no rows.
> wages.where(year == 1900).n_rows()
0
note: no row kept: the result is empty (line 4)

// Of the table `duplicates` gives: how many workers appear more than once.
> wages.duplicates(worker).n_rows()
3911
