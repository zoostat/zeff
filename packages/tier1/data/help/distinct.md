# distinct

## Summary
Rows with distinct values of the variables, the first occurrence kept

## Description
The rows whose values of the named variables are distinct: of the rows that share a value, the first in the current row order is kept, with every column
of the frame, and the rest are dropped, with a note of how many. Values are compared by
identity (section 3.7 of the Language Reference), so `NA` matches `NA` and rows with `NA` keys
keep one of their number. Sort first when the row to keep matters: `data.sort(year).distinct(firm)`
keeps each firm's earliest year. Like dplyr's `distinct(.keep_all = TRUE)` and Stata's
`duplicates drop`.

## Arguments
- data: The frame.
- vars: The variable, or list of variables, whose values must be distinct.

## Value
A frame of the kept rows, in their original order, with every column and the panel declaration.

## Details
Exact: the rows are grouped by the kernel's identity on the key values and the first row of each
group is kept; nothing is computed.

## See also
duplicates, unique, n_distinct, sort, drop_missing

## Examples
> let sales = frame(firm = [2, 1, 2, 1, 3], year = [2021, 2020, 2020, 2020, 2021], revenue = [12.5, 8, 11, 9, 7])
> sales.distinct(firm)
frame · 3 rows × 3 variables

      firm   year   revenue
  1      2   2021      12.5
  2      1   2020         8
  3      3   2021         7
note: 2 rows dropped as repeats of firm; 3 rows distinct (line 2)
> sales.distinct([firm, year])
frame · 4 rows × 3 variables

      firm   year   revenue
  1      2   2021      12.5
  2      1   2020         8
  3      2   2020        11
  4      3   2021         7
note: 1 row dropped as a repeat of firm, year; 4 rows distinct (line 3)
