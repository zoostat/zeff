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

## Missing values
A row whose later period is not in the data has `NA`: the last period of each unit, and the period before a gap. A value that is `NA` in the later period is `NA` in the lead.

## Details
Exact: each row takes the value of the row of its unit whose time is exactly `n` periods later,
found in the panel's sorted index.

## A common mistake
Calling it on a frame that is not a declared panel. Without a unit and a time there is no earlier or later period:

> dataset(firms).add(next = lead(size))
error[Z0602]: `lead` needs a panel, but firms has no declaration
  = help: declare the panel first: `data.panel(unit: firm, time: year)` (section 6 of the Language Reference)

Declare the panel first, `d.panel(unit: firm, time: year)`; the shipped firms have one row each and no time, so they have no lead.

## Coming from R or Stata
| d.add(next = lead(wage)) | dplyr::mutate(dplyr::group_by(dplyr::arrange(d, worker, year), worker), next = dplyr::lead(wage)) | generate next = F.wage |
| d.add(two = lead(wage, n: 2)) | dplyr::lead(wage, 2), in the same `mutate` | generate two = F2.wage |

dplyr's `lead` takes the next row whatever its time; here, as with Stata's `F.` after `xtset`,
the lead is the value one period later, and `NA` where that period is absent.

## Related tasks
- For the value one period earlier, use `lag`.
- For the change from the period before, use `diff`.
- To declare the unit and the time, use `panel`.

## See also
lag, diff, panel

## Examples
// Next year's wage of the same worker; the last year of each worker has none.
> let w = dataset(wages).where(worker <= 2).select([worker, year, wage])
> w.add(next = lead(wage))
frame · 11 rows × 4 variables · panel worker × year

       worker   year    wage    next
   1        1   2015   18.08   17.28
   2        1   2016   17.28   20.93
   3        1   2017   20.93   15.68
   4        1   2018   15.68   26.39
   5        1   2019   26.39   33.07
   6        1   2020   33.07   26.65
   7        1   2021   26.65      NA
   8        2   2014   40.67   27.38
   9        2   2015   27.38   35.68
  10        2   2016   35.68    22.2
  … and 1 more row

// Two periods ahead with `n:`, and whether the wage rises next year.
> w.add(two = lead(wage, n: 2), rises = lead(wage) > wage).head(4)
frame · 4 rows × 5 variables · panel worker × year

      worker   year    wage     two   rises
  1        1   2015   18.08   20.93   false
  2        1   2016   17.28   15.68   true
  3        1   2017   20.93   26.39   false
  4        1   2018   15.68   33.07   true
