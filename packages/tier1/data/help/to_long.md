# to_long

## Summary
Turn repeated columns into rows

## Description
The columns whose names begin with `stub` become rows: one row per original row and stub column,
each original row's stub columns in column order, with the rest of each column's name in the
column `names_to:` and its value in `values_to:`. `names_type:` says how the rest of the name
reads: as text (`Str`, the default), as a whole number (`Int`, so `sales_2010` gives the year
2010) or as a date written `2010-01-31` (`Date`). The `id:` variables and every other column
that is not a stub column are kept as they are, repeated on each new row, as tidyr's
`pivot_longer` keeps them. The stub columns share one type, Int joining Float; otherwise, or when
a name does not parse as `names_type:` says, the call is refused (Z0610). Like tidyr's
`pivot_longer` and Stata's `reshape long`.

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

## Details
Exact: values are moved; the names are parsed once. A panel declaration is dropped, with a note,
since the rows are reshaped.

## See also
to_wide, merge, append, rename

## Examples
> let sales = frame(firm = [1, 2], region = ["North", "South"], sales_2010 = [120, 80], sales_2011 = [130, NA])
> sales.to_long("sales_", id: firm, names_to: "year", values_to: "sales", names_type: Int)
frame · 4 rows × 4 variables

      firm   region   year   sales
  1      1   North    2010     120
  2      1   North    2011     130
  3      2   South    2010      80
  4      2   South    2011      NA
note: to_long: 2 columns of `sales_` into 4 rows as `year` and `sales` (line 2)
