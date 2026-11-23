# duplicates

## Summary
Table of repeated key values

## Description
The values of the named variables that more than one row holds, with the number of copies of each: a table titled `duplicates · firm_id` with the key
columns and `copies`, one row per repeated value in key order (`NA` last), in the shape of
Stata's `duplicates report`. A note says when no value is repeated. Values are compared by
identity, so rows with `NA` keys count together. Before a `merge` that expects one row per
key, `duplicates` shows which keys would break it; `distinct` then keeps the first of each.

The table is also data: `n_rows()` of it is the number of repeated values, 0 when there is
none, and `to(Frame)` gives it as a frame, the key variables with their own types and
`copies`, to count, filter or merge.

## Arguments
- data: The frame.
- vars: The variable, or list of variables, whose repeated values are counted.

## Value
A table with the key columns and `copies`; empty, with a note, when nothing is repeated.
`n_rows()` counts its rows and `to(Frame)` gives it as a frame.

## Missing values
`NA` is a value like any other here: the rows with `NA` in a variable named are counted together, in the last row of the table.

## Details
Exact: the rows are grouped by the kernel's identity on the key values and the groups of more
than one row are listed with their sizes.

## A common mistake
Calling it with no variable, as Stata's `duplicates report` does for whole observations. The variables to look at are named:

> dataset(wages).duplicates()
error[Z0406]: `duplicates` needs `vars`
  = help: usage: `data.duplicates(vars)`

Name the key: `dataset(wages).duplicates([worker, year])` shows whether any worker has two rows for one year.

## Coming from R or Stata
| d.duplicates(firm) | subset(as.data.frame(table(firm = d$firm)), Freq > 1) | duplicates report firm |

Stata's `duplicates report` counts how many observations have each number of copies; this table
lists the repeated values themselves, as `duplicates examples` does. R's `duplicated` marks the
second and later copies of each value and not the first.

## Related tasks
- To keep one row of each value, use `distinct`.
- For the number of different values, use `n_distinct`.
- For the count of every value, repeated or not, use `tabulate`.
- To join frames once the key is known to be clean, use `merge`.

## See also
distinct, n_distinct, merge

## Examples
// A panel has one row for each worker and year: no pair is repeated.
> let wages = dataset(wages)
> wages.duplicates([worker, year])
duplicates · worker, year

  worker   year   copies
note: no repeated values of worker, year (line 2)

// The values that repeat, with the number of rows of each.
> wages.where(worker <= 3).duplicates(worker)
duplicates · worker

  worker   copies
  1             7
  2             4
  3             2

// The table as data: how many firms have more than one row, and the three with the most.
> wages.duplicates(firm).n_rows()
> wages.duplicates(firm).to(Frame).sort(copies, desc: true).head(3)
397
frame · 3 rows × 2 variables

      firm   copies
  1     26      206
  2    268      204
  3    283      191
