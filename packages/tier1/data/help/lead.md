# lead

## Summary
Next values

## Description
Inside `add` or `update` of a declared panel, the value of `x` one period later for the same
unit, or `n:` periods later; `NA` where that period is missing, so a gap in time gives `NA` as
Stata's `F.` does. A period is a step of one for an Int time and one calendar day for a `Date`.
The result keeps the type and metadata of `x`; under `by:` the panel is read within each group.
A frame with no declaration, or a call outside `add` or `update`, is refused (Z0602).

## Arguments
- x: A variable of the frame, of any type.
- n: The periods to look ahead, a whole number of at least 1; 1 unless given.

## Value
A column of the type of `x`, one value per row.

## Details
Exact: each row takes the value of the row of its unit whose time is exactly `n` periods later,
found in the panel's sorted index.

## See also
lag, diff, panel

## Examples
> let p = frame(firm = [2, 1, 2, 1, 2], year = [2021, 2020, 2020, 2021, 2023], revenue = [12.5, 8, 11, 9, 15]).panel(unit: firm, time: year)
> p.add(next = lead(revenue))
frame · 5 rows × 4 variables · panel firm × year

      firm   year   revenue   next
  1      2   2021      12.5     NA
  2      1   2020         8      9
  3      2   2020        11   12.5
  4      1   2021         9     NA
  5      2   2023        15     NA
