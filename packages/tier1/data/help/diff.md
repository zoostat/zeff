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

## Missing values
The change is `NA` where the earlier period is not in the data, the first period of each unit and the period after a gap, and where either of the two values is `NA`.

## Details
Exact: the subtraction of the two values, row by row, with the lag found in the panel's sorted
index. Written in Rust rather than Zeff: its `lag` reads the panel of the frame around the
call, which a body written in Zeff does not pass on to the functions it calls (open decision
#69).

## A common mistake
Calling it on a frame that is not a declared panel. Without a unit and a time there is no earlier or later period:

> dataset(firms).add(change = diff(size))
error[Z0602]: `diff` needs a panel, but firms has no declaration
  = help: declare the panel first: `data.panel(unit: firm, time: year)` (section 6 of the Language Reference)

Declare the panel first, `d.panel(unit: firm, time: year)`; the shipped firms have one row each and no time, so nothing changes from one period to the next.

## Coming from R or Stata
| d.add(change = diff(wage)) | dplyr::mutate(dplyr::group_by(dplyr::arrange(d, worker, year), worker), change = wage - dplyr::lag(wage)) | generate change = D.wage |
| d.add(change = diff(wage, n: 2)) | wage - dplyr::lag(wage, 2), in the same `mutate` | generate change = S2.wage |

R's own `diff(x)` gives the differences of neighbouring values as a vector one shorter than
`x`; here every row has a value, `NA` where there is no earlier period. In Stata `D2.wage` is
the difference of the difference; the change over two periods, which `n: 2` gives here, is
`S2.wage`.

## Related tasks
- For the earlier value itself, use `lag`.
- For the later value, use `lead`.
- To declare the unit and the time, use `panel`.
- For a sum so far down the rows, use `running`.

## See also
lag, lead, panel

## Examples
// The change in a worker's wage from the year before.
> let w = dataset(wages).where(worker <= 2).select([worker, year, wage])
> w.add(change = diff(wage))
frame · 11 rows × 4 variables · panel worker × year

       worker   year    wage   change
   1        1   2015   18.08       NA
   2        1   2016   17.28     -0.8
   3        1   2017   20.93     3.65
   4        1   2018   15.68    -5.25
   5        1   2019   26.39    10.71
   6        1   2020   33.07     6.68
   7        1   2021   26.65    -6.42
   8        2   2014   40.67       NA
   9        2   2015   27.38   -13.29
  10        2   2016   35.68      8.3
  … and 1 more row

// Over two periods with `n:`.
> w.add(change2 = diff(wage, n: 2)).head(4)
frame · 4 rows × 4 variables · panel worker × year

      worker   year    wage   change2
  1        1   2015   18.08        NA
  2        1   2016   17.28        NA
  3        1   2017   20.93      2.85
  4        1   2018   15.68      -1.6

// A gap in time gives `NA`, where a difference of neighbouring rows would span two years.
> w.where(year != 2017).add(change = diff(wage)).head(4)
frame · 4 rows × 4 variables · panel worker × year

      worker   year    wage   change
  1        1   2015   18.08       NA
  2        1   2016   17.28     -0.8
  3        1   2018   15.68       NA
  4        1   2019   26.39    10.71
