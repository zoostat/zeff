# tabulate

## Summary
Frequencies and percentages, one- or two-way

## Description
One variable gives a table of its values with their counts (`N`) and the percentage of all rows
(`%`), then a `Total` row. Two variables give a cross-table: the first variable's values down the
side, the second's across the top, a `Total` column and a `Total` row. With `percent:`, each row
of counts is followed by a row of percentages: of its row (`row`), of its column (`col`) or of
every row of the frame (`total`); the `Total` row and column show the matching marginal
percentages.

`NA` is shown as its own value, last, so that the counts always add up to the rows of the frame;
R's `table` and Stata's `tabulate` hide it unless asked. Only the values present appear, in the
order of section 3.7 of the Language Reference (`false` before `true`, numbers numerically, text
by code point, dates in time order, categories by level); a labelled variable prints its labels.
Counts print with thousands separators and percentages with one decimal.

Like R's `table(x, y, useNA = "ifany")` with `prop.table`, and Stata's `tabulate` with
`missing` and its `row`, `column` and `cell` options.

## Arguments
- data: The frame.
- vars: One variable, `industry`, or two in a list, `[industry, female]`.
- percent: For two variables, the percentages shown under each row of counts: `row`, `col` or
  `total`. Omitted, counts alone. One variable's table always shows the percentage of all rows,
  so `row` and `col` are refused with one variable.

## Value
A `Table`, which prints as shown and can be passed to `print`.

## Details
Any variable that can group rows can be tabulated: Bool, Int, Float, Str, Date, DateTime or
Category; an opaque value cannot. A percentage with nothing to divide by (a frame with no rows)
prints `NA`. The table has no `weights:` in 1.0 (open decision #53 places frequency weights
at 1.1).

## References
Agresti (2013), Categorical Data Analysis, third edition, Wiley, chapter 2 (two-way contingency
tables and their marginal and conditional distributions).

## See also
summary, describe, aggregate, n_distinct

## Examples
> let wages = dataset(wages)
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
> let small = frame(sex = ["f", "m", NA, "f", "m", "f"], smoker = [true, false, true, NA, false, false])
> small.tabulate(sex)
tabulate · small · sex

  sex     N       %
  f       3    50.0
  m       2    33.3
  NA      1    16.7
  ─────────────────
  Total   6   100.0
> small.tabulate([sex, smoker])
tabulate · small · sex × smoker

  sex     false   true   NA   Total
  f           1      1    1       3
  m           2      0    0       2
  NA          0      1    0       1
  ─────────────────────────────────
  Total       3      2    1       6
