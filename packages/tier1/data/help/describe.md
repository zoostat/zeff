# describe

## Summary
Type, non-missing and missing count of each variable

## Description
One row per variable of the frame, in the frame's order: the variable's name, its type, how many
of its values are present, how many are missing, and an example. The example is the first value
that is not missing, printed as the value prints anywhere in Zeff (a Float to six significant
digits, a date in ISO form), so that the same file always shows the same example; a variable whose
values are all missing shows `NA`. A category's example adds its number of levels. The title line
is the frame's header, the same line that prints before a frame's first ten rows.

Like R's `str` and Stata's `describe`, with counts of missing values in place of storage details.

## Arguments
- data: The frame.

## Value
A `Table`, which prints as shown and can be passed to `print`.

## See also
summary, n_rows, n_cols, variables

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.describe()
wages · 3 rows × 3 variables

  variable   type    non-missing   missing   example
  id         Int               3         0   1
  wage       Float             3         0   23.4
  hours      Int               2         1   40
