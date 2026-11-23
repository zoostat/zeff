# lag

## Summary
Previous values in the panel's time order

## Description
Inside `add` or `update` of a declared panel, the value of `x` one period earlier for the same
unit, or `n:` periods earlier; `NA` where that period is missing, so a gap in time gives `NA` as
Stata's `L.` does and unlike dplyr's `lag`, which takes the previous row. A period is a step of
one for an Int time and one calendar day for a `Date`; the frame's row order plays no part. The
result keeps the type and metadata of `x`. Under `by:` the panel is read within each group. A
frame with no declaration, or a call outside `add` or `update`, is refused (Z0602).

## Arguments
- x: A variable of the frame, of any type.
- n: The periods to look back, a whole number of at least 1; 1 unless given.

## Value
A column of the type of `x`, one value per row.

## Missing values
A row whose earlier period is not in the data has `NA`: the first period of each unit, and the period after a gap. A value that is `NA` in the earlier period is `NA` in the lag.

## Details
Exact: each row takes the value of the row of its unit whose time is exactly `n` periods earlier,
found in the panel's sorted index.

## A common mistake
Calling it on a frame that is not a declared panel. Without a unit and a time there is no earlier or later period:

> dataset(firms).add(prev = lag(size))
error[Z0602]: `lag` needs a panel, but firms has no declaration
  = help: declare the panel first: `data.panel(unit: firm, time: year)` (section 6 of the Language Reference)

Declare the panel first, `d.panel(unit: firm, time: year)`; the shipped firms have one row each and no time, so they have no lag.

## Coming from R or Stata
| d.add(prev = lag(wage)) | dplyr::mutate(dplyr::group_by(dplyr::arrange(d, worker, year), worker), prev = dplyr::lag(wage)) | generate prev = L.wage |
| d.add(two = lag(wage, n: 2)) | dplyr::lag(wage, 2), in the same `mutate` | generate two = L2.wage |

dplyr's `lag` takes the previous row whatever its time, so the data are sorted and grouped
first and a missing year gives the value of two years before. Here, as with Stata's `L.` after
`xtset`, the lag is the value one period earlier, and `NA` where that period is absent.

## Related tasks
- For the value one period later, use `lead`.
- For the change from the period before, use `diff`.
- To declare the unit and the time, use `panel`.
- To carry the last value forward into a gap, use `fill_missing`.

## See also
lead, diff, fill_missing, panel

## Examples
// Last year's wage of the same worker; the first year of each worker has none.
> let w = dataset(wages).where(worker <= 2).select([worker, year, wage])
> w.add(prev = lag(wage))
frame · 11 rows × 4 variables · panel worker × year

       worker   year    wage    prev
   1        1   2015   18.08      NA
   2        1   2016   17.28   18.08
   3        1   2017   20.93   17.28
   4        1   2018   15.68   20.93
   5        1   2019   26.39   15.68
   6        1   2020   33.07   26.39
   7        1   2021   26.65   33.07
   8        2   2014   40.67      NA
   9        2   2015   27.38   40.67
  10        2   2016   35.68   27.38
  … and 1 more row

// Two periods back with `n:`, and a growth rate.
> w.add(two = lag(wage, n: 2), growth = wage / lag(wage) - 1).head(4)
frame · 4 rows × 5 variables · panel worker × year

      worker   year    wage     two       growth
  1        1   2015   18.08      NA           NA
  2        1   2016   17.28      NA   -0.0442478
  3        1   2017   20.93   18.08     0.211227
  4        1   2018   15.68   17.28    -0.250836

// A gap in time gives `NA`: without its row of 2017, the worker has no lag in 2018.
> w.where(year != 2017).add(prev = lag(wage)).head(4)
frame · 4 rows × 4 variables · panel worker × year

      worker   year    wage    prev
  1        1   2015   18.08      NA
  2        1   2016   17.28   18.08
  3        1   2018   15.68      NA
  4        1   2019   26.39   15.68
