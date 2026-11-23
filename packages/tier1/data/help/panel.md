# panel

## Summary
Declare unit and time variables

## In brief
- `d.panel(unit: firm, time: year)`: the frame declared a panel of firms observed by year.
- After it `lag`, `lead` and `diff` follow each unit's time, and the frame's header shows the declaration.

## Description
Declares the frame a panel: `unit:` names the variable that identifies a unit (a worker, a firm,
a county) and `time:` the variable that orders its observations, an Int or a `Date`. After it,
`lag`, `lead`, `diff` and `fill_missing` with `method:` follow each unit's time order and `did` takes its unit and
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

## Missing values
`NA` in the unit or in the time variable is refused (Z0601), with the rows named: a row with no unit or no time has no place in a panel. Missing values in the other variables do not matter to the declaration.

## Details
Exact: the index is the stable sort of section 6 by unit then time, so every target orders the
rows the same way.

## A common mistake
Naming a unit that has several rows at one time. In the shipped wages a firm has many workers in a year, so a firm is not the unit of these rows:

> dataset(wages).panel(unit: firm, time: year)
error[Z0601]: `firm` 1 is observed twice at `year` 2014; 2,970 other pairs repeat
  = help: a unit has one row per time: `duplicates([firm, year])` lists the repeated pairs; drop or aggregate them first

Name the variable that has one row at each time, `unit: worker`, or summarise first: `aggregate(wage = mean(wage), by: [firm, year])` has one row for each firm and year.

## Coming from R or Stata
| d.panel(unit: firm, time: year) | plm::pdata.frame(d, index = c("firm", "year")), or tsibble::as_tsibble(d, key = firm, index = year) | xtset firm year |

Base R and dplyr have no declaration: `dplyr::lag` takes the previous row, so the data are
sorted and grouped by hand before each lag. Stata's `xtset` asks for a unit that is a number;
here the unit may be text or a category.

## Related tasks
- For the value one period earlier, use `lag`.
- For the value one period later, use `lead`.
- For the change from the period before, use `diff`.
- To see which pairs of unit and time repeat, use `duplicates`.
- To fill a missing value from the period before, use `fill_missing`.

## See also
describe, sort, where

## Examples
// The shipped wages come declared, and the header says so.
> dataset(wages).head(2)
frame · 2 rows × 11 variables · panel worker × year

      worker   year    wage   educ   exper   age   hours   union   female   industry   firm
  1        1   2015   18.08     17       9    32    45.1   no      male     Retail       99
  2        1   2016   17.28     17      10    33    41.4   no      male     Retail       99

// A declaration made: the wages summarised to one row for each firm and year, as a panel of firms.
> let firms = dataset(wages).aggregate(mean_wage = mean(wage), workers = n(), by: [firm, year])
> let p = firms.panel(unit: firm, time: year)
> p.head(3)
warning[W0201]: 211 rows with NA in wage were skipped (line 2)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 2)
frame · 3 rows × 4 variables · panel firm × year

      firm   year   mean_wage   workers
  1      1   2013       26.28         1
  2      1   2014       20.12         2
  3      1   2015        32.8         2

// After it, `lag` follows each firm's years.
> p.add(prev = lag(mean_wage)).head(4)
frame · 4 rows × 5 variables · panel firm × year

      firm   year   mean_wage   workers    prev
  1      1   2013       26.28         1      NA
  2      1   2014       20.12         2   26.28
  3      1   2015        32.8         2   20.12
  4      1   2016        27.4         2    32.8

// The declaration survives a verb that keeps both variables, and is renamed with them.
> p.where(year > 2015).rename(company = firm).head(2)
frame · 2 rows × 4 variables · panel company × year

      company   year   mean_wage   workers
  1         1   2016        27.4         2
  2         1   2017       40.24         1

// A verb that leaves one of them out drops it, with a note.
> p.drop(year).head(2)
frame · 2 rows × 3 variables

      firm   mean_wage   workers
  1      1       26.28         1
  2      1       20.12         2
note: the panel declaration `firm × year` dropped: `drop` left out `year` (line 7)
