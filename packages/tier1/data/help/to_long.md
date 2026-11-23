# to_long

## Summary
Turn repeated columns into rows

## In brief
- `d.to_long("sales_", id: firm, names_to: "year", values_to: "sales")`: the columns `sales_2010`, `sales_2011`, … become rows, the rest of each name in `year` as text.
- `names_type: Int` reads the rest of each name as a whole number; `names_type: Date` reads it as a date.

## Description
The columns whose names begin with `stub` become rows: one row per original row and stub column,
each original row's stub columns in column order, with the rest of each column's name in the
column `names_to:` and its value in `values_to:`. `names_type:` says how the rest of the name
reads: as text (`Str`, the default), as a whole number (`Int`, so `sales_2010` gives the year
2010) or as a date written `2010-01-31` (`Date`). The `id:` variables and every other column
that is not a stub column are kept as they are, repeated on each new row, as tidyr's
`pivot_longer` keeps them. The stub columns share one type, Int joining Float; otherwise, or when
a name does not parse as `names_type:` says, the call is refused (Z0610).

## Arguments
- data: The frame.
- stub: The start the repeated names share, `"sales_"` for `sales_2010`, `sales_2011`, ...
- id: The variable, or list of variables, that identifies a row; kept, like every other column that is not a stub column.
- names_to: The name of the new column holding the rest of each old name.
- values_to: The name of the new column holding the values.
- names_type: `Str`, `Int` or `Date`, how the rest of each name reads; `Str` unless given.

## Value
A frame with the kept columns, then `names_to:` and `values_to:`, one row per original row and
stub column.

## Missing values
A value that is `NA` in a stub column is a row with `NA` in `values_to:`; no row is dropped.

## Details
Exact: values are moved; the names are parsed once. A panel declaration is dropped, with a note,
since the rows are reshaped.

## A common mistake
Saying that the rest of each name is a date when it is a year. The names must read as `names_type:` says:

> wide.to_long("wage_", id: worker, names_to: "year", values_to: "wage", names_type: Date)
error[Z0610]: `wage_2019` does not end in a date written as 2010-01-31: `2019` after `wage_`
  = help: `names_type: Str` keeps the rest of each name as text; `rename` a column whose name is wrong

A year is a whole number: `names_type: Int`.

## Coming from R or Stata
| d.to_long("sales_", id: firm, names_to: "year", values_to: "sales", names_type: Int) | tidyr::pivot_longer(d, starts_with("sales_"), names_to = "year", names_prefix = "sales_", values_to = "sales", names_transform = as.integer) | reshape long sales_, i(firm) j(year) |

In Stata the stub is written without quotation marks and `j()` names the new variable; a rest
of the name that is not a number needs the option `string`. tidyr leaves the rest of the name
as text unless `names_transform` is given; here `names_type:` says how it reads.

## Related tasks
- For the other direction, rows into columns, use `to_wide`.
- To put the rows of two frames one under the other, use `append`.
- To give the columns names with one stub first, use `rename`.

## See also
to_wide, merge, append, rename

## Examples
// A wide frame made from the shipped wages, one column for each year, and back to one row for each worker and year.
> let some = dataset(wages).where((worker == 1 || worker == 15 || worker == 17) && year >= 2019).select([worker, year, wage])
> let wide = some.to_wide(values_from: wage, id: worker, names_from: year)
> wide
> wide.to_long("wage_", id: worker, names_to: "year", values_to: "wage", names_type: Int)
note: to_wide: 3 columns of `wage` by `year`, 3 rows of `worker` (line 2)
note: the panel declaration `worker × year` dropped: `to_wide` reshaped the rows (line 2)
wide · 3 rows × 4 variables

      worker   wage_2019   wage_2020   wage_2021
  1        1       26.39       33.07       26.65
  2       15       20.14       26.73       28.21
  3       17       49.23       20.96          NA
frame · 9 rows × 3 variables

      worker   year    wage
  1        1   2019   26.39
  2        1   2020   33.07
  3        1   2021   26.65
  4       15   2019   20.14
  5       15   2020   26.73
  6       15   2021   28.21
  7       17   2019   49.23
  8       17   2020   20.96
  9       17   2021      NA
note: to_long: 3 columns of `wage_` into 9 rows as `year` and `wage` (line 4)

// The rest of the name kept as text, which is the default.
> wide.to_long("wage_", id: worker, names_to: "period", values_to: "wage").head(2)
frame · 2 rows × 3 variables

      worker   period    wage
  1        1   2019     26.39
  2        1   2020     33.07
note: to_long: 3 columns of `wage_` into 9 rows as `period` and `wage` (line 5)
