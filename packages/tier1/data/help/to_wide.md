# to_wide

## Summary
Turn rows into repeated columns

## Description
The inverse of `to_long`: one row per value of the `id:` variables, in order of first
appearance, and one new column per value of `names_from:`, named `<values_from>_<value>`
(`sales_2010`), in the order the values sort, holding the value of `values_from:` for that id
and name; a combination that no row holds is `NA`, and two rows with one id and one name are
refused (Z0611), since one cell cannot hold both. A column that is neither an id, the names nor
the values is left out, and the note says so; name it in `id:` to keep it. A new column whose
name is not a valid name, `sales_North East`, is written in backticks in a script. The panel
declaration is dropped, with a note. Like tidyr's `pivot_wider` and Stata's `reshape wide`.

## Arguments
- data: The frame, one row per id and name.
- values_from: The variable whose values fill the new columns.
- id: The variable, or list of variables, that identifies a row of the result.
- names_from: The variable whose values name the new columns.

## Value
A frame with the `id:` variables, then one column per value of `names_from:`.

## Details
Exact: values are moved; the ids are grouped by identity (section 3.7 of the Language
Reference), so `NA` is an id like any other, and the names sort as `sort` sorts them, `NA` last.

## See also
to_long, merge, aggregate

## Examples
> let long = frame(firm = [1, 1, 2, 2, 3], year = [2010, 2011, 2010, 2011, 2011], sales = [120, 130, 80, NA, 95])
> long.to_wide(values_from: sales, id: firm, names_from: year)
frame · 3 rows × 3 variables

      firm   sales_2010   sales_2011
  1      1          120          130
  2      2           80           NA
  3      3           NA           95
note: to_wide: 2 columns of `sales` by `year`, 3 rows of `firm` (line 2)
