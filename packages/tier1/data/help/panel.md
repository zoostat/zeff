# panel

## Summary
Declare unit and time variables

## Description
Declares the frame a panel: `unit:` names the variable that identifies a unit (a worker, a firm,
a county) and `time:` the variable that orders its observations, an Int or a `Date`. After it,
`lag`, `lead`, `diff` and `fill_forward` follow each unit's time order and `did` takes its unit and
time from the panel, so none of them repeats `unit:` and `time:`; the declaration prints in the
frame's header, `panel worker × year`, and is kept in Parquet and Arrow files under the Zeff
Parquet profile. The frame is checked as Stata's `xtset` checks it: a unit observed twice at one
time, or `NA` in either variable, is refused (Z0601) with the rows named, since a panel's lags
would be ambiguous. The rows keep their order; the sorted index the panel functions use is built
once and kept with the frame. The declaration survives a verb that keeps both variables unchanged
(`where`, `sort`, `add`, `select` with both among them, `rename` of either, which renames the
declaration too) and is dropped with a note by one that does not (`aggregate`, `to_wide`, a `drop`
or `select` that leaves one out, an `add`, `update` or `recode` that changes either). A gap in
time gives `NA` from the panel functions: consecutive integers for an Int, consecutive days for a
`Date`; a monthly or quarterly panel uses an Int index such as `year * 12 + month` until the
`Period` type arrives (section 6 of the Language Reference). `unit:` takes one variable in 1.0.

## Arguments
- data: The frame.
- unit: The variable that identifies a unit, of any type but an opaque one.
- time: The variable that orders a unit's rows: an Int or a `Date`.

## Value
A `Frame` with the same rows and variables and the panel declaration.

## Details
Exact: the index is the stable sort of section 6 by unit then time, so every target orders the
rows the same way.

## See also
describe, sort, where

## Examples
> let sales = frame(firm = [2, 1, 2, 1], year = [2021, 2020, 2020, 2021], revenue = [12.5, 8, 11, 9])
> let p = sales.panel(unit: firm, time: year)
> p
p · 4 rows × 3 variables · panel firm × year

      firm   year   revenue
  1      2   2021      12.5
  2      1   2020         8
  3      2   2020        11
  4      1   2021         9
> p.where(year > 2020).rename(company = firm)
frame · 2 rows × 3 variables · panel company × year

      company   year   revenue
  1         2   2021      12.5
  2         1   2021         9
> p.drop(year)
frame · 4 rows × 2 variables

      firm   revenue
  1      2      12.5
  2      1         8
  3      2        11
  4      1         9
note: the panel declaration `firm × year` dropped: `drop` left out `year` (line 5)
> sales.panel(unit: firm, time: revenue)
error[Z0601]: `time:` of `panel` must be an Int or a Date, but `revenue` is a Float variable
 --> examples.zeff:6:31
  |
6 | sales.panel(unit: firm, time: revenue)
  |                               ^^^^^^^ not a period
  |
  = help: a panel's time is a whole-number period or a calendar day; `to(revenue, Int)` if the values are whole
