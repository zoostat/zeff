# fill_forward

## Summary
Replace NA with the last non-missing value, within panel units or by: groups

## Description
Inside `add` or `update`, each `NA` of `x` replaced by the last value present before it: within
each unit in the panel's time order when the frame is a declared panel, within each group in
row order under `by:`, and in the frame's row order otherwise. An `NA` before the first value of
its unit or group stays `NA`. The result keeps the type and metadata of `x`. Like tidyr's `fill`
and Stata's `carryforward`. A call outside `add` or `update` is refused (Z0602).

## Arguments
- x: A variable of the frame, of any type.

## Value
A column of the type of `x`, one value per row.

## Details
Exact: one pass over the rows in the order described, carrying the last row that held a value.

## See also
lag, coalesce, fill_missing, panel

## Examples
> let p = frame(firm = [1, 1, 1, 2, 2], year = [2020, 2021, 2022, 2020, 2021], price = [NA, 10, NA, NA, 7]).panel(unit: firm, time: year)
> p.add(carried = fill_forward(price))
frame · 5 rows × 4 variables · panel firm × year

      firm   year   price   carried
  1      1   2020      NA        NA
  2      1   2021      10        10
  3      1   2022      NA        10
  4      2   2020      NA        NA
  5      2   2021       7         7
> frame(g = [1, 1, 2, 2], v = [5, NA, 7, NA]).add(f = fill_forward(v), by: g)
frame · 4 rows × 3 variables

      g    v   f
  1   1    5   5
  2   1   NA   5
  3   2    7   7
  4   2   NA   7
