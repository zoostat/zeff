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

## Details
Exact: the cells are the values' printed forms; nothing is computed.

## See also
export, summary, aggregate, frame

## Examples
> table(title: "sales by year", columns: {year: [2010, 2011, 2012], sales: [120.5, 130, NA], region: ["North", "South", "West"]})
sales by year

  year   sales   region
  2010   120.5   North
  2011     130   South
  2012      NA   West
> let w = dataset(wages).where(worker <= 2)
> table(title: "two workers", columns: {worker: w["worker"], year: w["year"], wage: w["wage"], female: w["female"]})
two workers

  worker   year    wage   female
       1   2015   18.08     male
       1   2016   17.28     male
       1   2017   20.93     male
       1   2018   15.68     male
       1   2019   26.39     male
       1   2020   33.07     male
       1   2021   26.65     male
       2   2014   40.67   female
       2   2015   27.38   female
       2   2016   35.68   female
       2   2017    22.2   female
