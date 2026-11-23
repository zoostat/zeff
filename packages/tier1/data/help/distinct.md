# distinct

## Summary
Rows with distinct values of the variables, the first occurrence kept

## Description
The rows whose values of the named variables are distinct: of the rows that share a value, the first in the current row order is kept, with every column
of the frame, and the rest are dropped, with a note of how many. Values are compared by
identity (section 3.7 of the Language Reference), so `NA` matches `NA` and rows with `NA` keys
keep one of their number. Sort first when the row to keep matters: `data.sort(year).distinct(firm)`
keeps each firm's earliest year.

## Arguments
- data: The frame.
- vars: The variable, or list of variables, whose values must be distinct.

## Value
A frame of the kept rows, in their original order, with every column and the panel declaration.

## Missing values
`NA` is a value like any other here: the rows with `NA` in a variable named count as repeats of one another, and one of them is kept.

## Details
Exact: the rows are grouped by the kernel's identity on the key values and the first row of each
group is kept; nothing is computed.

## A common mistake
Calling it with no variable, as dplyr's `distinct()` does for rows alike in everything. The variables whose values must differ are named:

> dataset(firms).distinct()
error[Z0406]: `distinct` needs `vars`
  = help: usage: `data.distinct(vars)`

Name them: `dataset(firms).distinct(industry)`; for rows alike in every variable, name them all with a range, `distinct(firm..founded)`.

## Coming from R or Stata
| d.distinct(firm) | d[!duplicated(d$firm), ], or dplyr::distinct(d, firm, .keep_all = TRUE) | duplicates drop firm, force |
| d.distinct([firm, year]) | dplyr::distinct(d, firm, year, .keep_all = TRUE) | duplicates drop firm year, force |

dplyr's `distinct(d, firm)` keeps the variable named and drops the others unless
`.keep_all = TRUE`; here every column is kept. Stata's `duplicates drop` with variables asks for
`force`, since rows alike on them may differ elsewhere; here the note counts the rows that went.

## Related tasks
- To see which values repeat before dropping any, use `duplicates`.
- For the number of different values, use `n_distinct`.
- For the different values of one variable as a list, use `unique`.
- To choose which of the repeats is kept, sort first with `sort`.

## See also
duplicates, unique, n_distinct, sort, drop_missing

## Examples
// One row for each worker of the shipped wages: the first of each is kept, with every column.
> let wages = dataset(wages)
> wages.distinct(worker).n_rows()
3911
note: 14,501 rows dropped as repeats of worker; 3,911 rows distinct (line 2)

// Distinct pairs of two variables.
> wages.distinct([firm, year]).n_rows()
4072
note: 14,340 rows dropped as repeats of firm, year; 4,072 rows distinct (line 3)

// Sort first when the row to keep matters: each worker's latest year.
> wages.sort(year, desc: true).distinct(worker).select([worker, year, wage]).head(3)
frame · 3 rows × 3 variables · panel worker × year

      worker   year    wage
  1        1   2021   26.65
  2       15   2021   28.21
  3       23   2021   37.79
note: 14,501 rows dropped as repeats of worker; 3,911 rows distinct (line 4)

// Rows with `NA` in the variable are repeats of one another.
> frame(id = [1, 2, 3, 4], region = ["North", NA, "North", NA]).distinct(region)
frame · 2 rows × 2 variables

      id   region
  1    1   North
  2    2   NA
note: 2 rows dropped as repeats of region; 2 rows distinct (line 5)
