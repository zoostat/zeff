# diff

## Summary
x − lag(x, n), so it respects the panel's time order and gaps

## Description
Inside `add` or `update` of a declared panel, the change in `x` from `n:` periods earlier for
the same unit: `x - lag(x, n)`, so a gap in time gives `NA`, as Stata's `D.` does, and the
frame's row order plays no part. An Int variable gives an Int difference (Z0211 on overflow) and
a Float variable a Float one. A frame with no declaration, or a call outside `add` or `update`,
is refused (Z0602).

## Arguments
- x: An Int or a Float variable of the frame.
- n: The periods to look back, a whole number of at least 1; 1 unless given.

## Value
A column of the type of `x`, one value per row.

## Details
Exact: the subtraction of the two values, row by row, with the lag found in the panel's sorted
index. Written in Rust rather than Zeff: its `lag` reads the panel of the frame around the
call, which a body written in Zeff does not pass on to the functions it calls (open decision
#69).

## See also
lag, lead, panel

## Examples
> let p = frame(firm = [2, 1, 2, 1, 2], year = [2021, 2020, 2020, 2021, 2023], revenue = [12.5, 8, 11, 9, 15]).panel(unit: firm, time: year)
> p.add(change = diff(revenue))
frame · 5 rows × 4 variables · panel firm × year

      firm   year   revenue   change
  1      2   2021      12.5      1.5
  2      1   2020         8       NA
  3      2   2020        11       NA
  4      1   2021         9        1
  5      2   2023        15       NA
