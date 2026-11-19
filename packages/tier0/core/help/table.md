# table

## Summary
A table from named lists or columns

## Description
Builds a `Table` value, the kind every statistic and `summary` return, from a record whose
fields are its columns: each field a list of values or a column of a frame, all of one length,
in the order written. Numbers are right-aligned and printed by the display rules of section 3.8
(a Float to six significant digits); text, Bools and dates are left-aligned, and a frame's column
with value labels shows its labels. To print numbers with a chosen number of decimals, format
them first with `format` and pass the texts. The table prints with its title, exports as Word or
LaTeX with `export`, and gives a function written in Zeff a table of its own; `aggregate` gives a
frame, not a table, when the rows are data to compute with.

## Arguments
- title: The line printed above the grid.
- columns: A record whose fields are the columns, `{year: years, mean: means}`, each a list or a column of one length.

## Value
A `Table` with one column per field of `columns:`.

## Missing values
A missing value is shown as `NA` in its cell; no row is dropped.

## Details
Exact: the cells are the values' printed forms; nothing is computed.

## A common mistake
Giving columns of different lengths. Every column has one value for each row, and the message says which differ:

> table(title: "t", columns: {a: [1, 2], b: [1]})
error[Z0401]: the columns of `table` differ in length: `a` has 2 rows and `b` has 1 row
  = help: every column of a table has one value per row

Give every column the same number of values, with `NA` where one is not known.

## Coming from R or Stata
| table(title: "t", columns: {a: xs, b: ys}) | data.frame(a = xs, b = ys), printed | list a b |

R's `table` is another thing, a count of values: that is `tabulate` here. Stata lists variables
of the data in memory; it has no table built from loose values.

## Related tasks
- For counts of a variable's values, use `tabulate`.
- To write a table to a file, use `export`.

## See also
export, summary, aggregate, frame

## Examples
// A table from lists; numbers to the right, text to the left, a missing value shown.
> table(title: "sales by year", columns: {year: [2010, 2011, 2012], sales: [120.5, 130, NA], region: ["North", "South", "West"]})
sales by year

  year   sales   region
  2010   120.5   North
  2011     130   South
  2012      NA   West

// A table from columns of the shipped wages; the labelled variable shows its labels.
> let w = dataset(wages).where(worker == 19)
> table(title: "one worker", columns: {year: w["year"], wage: w["wage"], female: w["female"]})
one worker

  year    wage   female
  2012   12.64     male
  2013   25.18     male
  2014   35.49     male
  2015   26.07     male
  2016      NA     male
