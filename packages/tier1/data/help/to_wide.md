# to_wide

## Summary
Turn rows into repeated columns

## In brief
- `d.to_wide(values_from: sales, id: firm, names_from: year)`: one row for each firm, one column `sales_<year>` for each year.
- `id: [firm, region]`: several variables identify a row, and each is kept.

## Description
The inverse of `to_long`: one row per value of the `id:` variables, in order of first
appearance, and one new column per value of `names_from:`, named `<values_from>_<value>`
(`sales_2010`), in the order the values sort, holding the value of `values_from:` for that id
and name; a combination that no row holds is `NA`, and two rows with one id and one name are
refused (Z0611), since one cell cannot hold both. A column that is neither an id, the names nor
the values is left out, and the note says so; name it in `id:` to keep it. A new column whose
name is not a valid name, `sales_North East`, is written in backticks in a script. The panel
declaration is dropped, with a note.

## Arguments
- data: The frame, one row per id and name.
- values_from: The variable whose values fill the new columns.
- id: The variable, or list of variables, that identifies a row of the result.
- names_from: The variable whose values name the new columns.

## Value
A frame with the `id:` variables, then one column per value of `names_from:`.

## Missing values
A combination of an id and a name that no row holds is `NA` in the new column. A value that is `NA` stays `NA`. An id that is `NA` is an id like any other, and a name that is `NA` gives a column of its own, the last.

## Details
Exact: values are moved; the ids are grouped by identity (section 3.7 of the Language
Reference), so `NA` is an id like any other, and the names sort as `sort` sorts them, `NA` last.

## A common mistake
Giving an id that does not identify a row for each name: two rows would go into one cell. Here a firm has many workers in a year:

> dataset(wages).to_wide(values_from: wage, id: firm, names_from: year)
error[Z0611]: `firm` 57 appears twice at `year` 2014
  = help: `duplicates([firm, year])` lists the repeated pairs; `aggregate` them or add the variable that tells them apart to `id:`

Give the id that has one row for each name, `id: worker`, or summarise first: `aggregate(wage = mean(wage), by: [firm, year])`.

## Coming from R or Stata
| d.to_wide(values_from: sales, id: firm, names_from: year) | tidyr::pivot_wider(d, id_cols = firm, names_from = year, values_from = sales, names_prefix = "sales_") | reshape wide sales, i(firm) j(year) |

Stata names the new variables `sales2010`; here, and in tidyr with the prefix, they are
`sales_2010`. Where two rows share an id and a name, `pivot_wider` puts both values into one
cell as a list and warns; here that is refused, and the message names the first such pair.

## Related tasks
- For the other direction, columns into rows, use `to_long`.
- To make one row for each id and name first, use `aggregate`.
- To see which pairs of id and name repeat, use `duplicates`.

## See also
to_long, merge, aggregate

## Examples
// One row for each worker, one column for each year; a year a worker has no row for is `NA`.
> let w = dataset(wages).where((worker == 1 || worker == 15 || worker == 17) && year >= 2019).select([worker, year, wage, educ])
> w.to_wide(values_from: wage, id: worker, names_from: year)
frame · 3 rows × 4 variables

      worker   wage_2019   wage_2020   wage_2021
  1        1       26.39       33.07       26.65
  2       15       20.14       26.73       28.21
  3       17       49.23       20.96          NA
note: to_wide: 3 columns of `wage` by `year`, 3 rows of `worker`; `educ` left out: name it in `id:` to keep it (line 2)
note: the panel declaration `worker × year` dropped: `to_wide` reshaped the rows (line 2)

// A variable that is neither an id, the names nor the values is left out, and the note says so; named in `id:` it is kept.
> w.to_wide(values_from: wage, id: [worker, educ], names_from: year)
frame · 3 rows × 5 variables

      worker   educ   wage_2019   wage_2020   wage_2021
  1        1     17       26.39       33.07       26.65
  2       15     17       20.14       26.73       28.21
  3       17     16       49.23       20.96          NA
note: to_wide: 3 columns of `wage` by `year`, 3 rows of `worker, educ` (line 3)
note: the panel declaration `worker × year` dropped: `to_wide` reshaped the rows (line 3)
