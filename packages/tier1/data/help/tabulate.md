# tabulate

## Summary
Frequencies and percentages, one- or two-way

## In brief
- `wages.tabulate(industry)`: counts and percentages of one variable, with a total.
- `wages.tabulate([female, union])`: a cross-table of counts.
- `wages.tabulate([female, union], percent: row)`: the same, with the percentage of each row
  under its counts.

## Description
One variable gives a table of its values with their counts (`N`) and the percentage of all rows
(`%`), then a `Total` row. Two variables give a cross-table: the first variable's values down the
side, the second's across the top, a `Total` column and a `Total` row. With `percent:`, each row
of counts is followed by a row of percentages: of its row (`row`), of its column (`col`) or of
every row of the frame (`total`); the `Total` row and column show the matching marginal
percentages.

Only the values present appear, in the order of section 3.7 of the Language Reference (`false`
before `true`, numbers numerically, text by code point, dates in time order, categories by
level); a labelled variable prints its labels. Counts print with thousands separators and
percentages with one decimal.

## Arguments
- data: The frame.
- vars: One variable, `industry`, or two in a list, `[industry, female]`.
- percent: For two variables, the percentages shown under each row of counts: `row`, `col` or
  `total`. Omitted, counts alone. One variable's table always shows the percentage of all rows,
  so `row` and `col` are refused with one variable.

## Value
A `Table`, which prints as shown and can be passed to `print`.

## Missing values
`NA` is shown as a value of its own, in the last row or column, so that the counts always add up
to the rows of the frame; no row is dropped and no note is printed. R's `table` and Stata's
`tabulate` show it only when asked, with `useNA = "ifany"` and `missing`.

## Details
### What can be tabulated
Any variable that can group rows can be tabulated: Bool, Int, Float, Str, Date, DateTime or
Category; an opaque value cannot. A percentage with nothing to divide by (a frame with no rows)
prints `NA`. The table has no `weights:` in 1.0 (open decision #53 places frequency weights
at 1.1).

## A common mistake
Asking for row percentages of one variable. A one-way table already shows each value's share of
all rows, so `percent:` is refused there, and the message gives the two ways out:

> wages.tabulate(industry, percent: row)
error[Z0401]: `percent: row` and `percent: col` need two variables; one variable's table always shows the percentage of all rows
  = help: give two variables, `tabulate([industry, female], percent: row)`, or leave `percent:` out

Leave `percent:` out for one variable, or give two: `wages.tabulate([industry, female], percent:
row)`.

## Coming from R or Stata
| tabulate(industry) | table(d$industry, useNA = "ifany") | tabulate industry, missing |
| tabulate([female, union], percent: row) | prop.table(table(d$female, d$union), 1) | tabulate female union, row |
| percent: col | prop.table(table(d$female, d$union), 2) | tabulate female union, column |
| percent: total | prop.table(table(d$female, d$union)) | tabulate female union, cell |

Missing values are counted without asking, as the section above says. A labelled variable prints
its labels, as Stata does; R prints the codes unless the variable is a factor. Percentages print
with one decimal, and the `Total` row and column carry the marginal percentages.

## Related tasks
- To test whether two variables are independent, use `chi2_test`.
- For counts by group as a frame, use `aggregate`.
- For means and spreads instead of counts, use `summary`.
- To count the distinct values, use `n_distinct`.

## References
Agresti (2013), Categorical Data Analysis, third edition, Wiley, chapter 2 (two-way contingency
tables and their marginal and conditional distributions).

## See also
summary, describe, aggregate, n_distinct, chi2_test

## Examples
// A labelled variable: its labels, its counts and each one's share of all rows.
> let wages = dataset(wages)
> wages.tabulate(union)
tabulate · wages · union

  union        N       %
  no      12,556    68.2
  yes      5,856    31.8
  ──────────────────────
  Total   18,412   100.0

// A category with twelve levels, in the order of its levels.
> wages.tabulate(industry)
tabulate · wages · industry

  industry                     N       %
  Agriculture                535     2.9
  Mining                     200     1.1
  Utilities                  370     2.0
  Construction             1,477     8.0
  Manufacturing            2,574    14.0
  Retail                   2,871    15.6
  Transport                  976     5.3
  Finance                  1,560     8.5
  Services                 2,637    14.3
  Education                1,693     9.2
  Health                   2,098    11.4
  Public administration    1,421     7.7
  ──────────────────────────────────────
  Total                   18,412   100.0

// A cross-table with the percentage of each row under its counts.
> wages.tabulate([female, union], percent: row)
tabulate · wages · female × union · row %

  female       no     yes    Total
  male      6,516   2,975    9,491
             68.7    31.3    100.0
  female    6,040   2,881    8,921
             67.7    32.3    100.0
  ────────────────────────────────
  Total    12,556   5,856   18,412
             68.2    31.8    100.0

// Missing values are a value of their own, last, so that the counts add up to the rows.
> let small = frame(sex = ["f", "m", NA, "f", "m", "f"], smoker = [true, false, true, NA, false, false])
> small.tabulate([sex, smoker])
tabulate · small · sex × smoker

  sex     false   true   NA   Total
  f           1      1    1       3
  m           2      0    0       2
  NA          0      1    0       1
  ─────────────────────────────────
  Total       3      2    1       6
