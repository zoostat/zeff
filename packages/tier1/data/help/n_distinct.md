# n_distinct

## Summary
Number of distinct values

## In brief
- `d.aggregate(firms = n_distinct(firm), by: year)`: the number of different values in each group.
- `d.add(k = n_distinct(firm), by: industry)`: the same count, on every row of the group.
- `n_distinct(d["firm"])`: of a whole column.

## Description
The number of distinct values of a variable, by identity (section 3.7 of the Language Reference):
`NA` counts as one value, and `-0.0` is `0`. As a reducer inside `add` or `update`, it counts within
the frame or, under `by:`, within the group; on a column written `wages["region"]` it counts the
whole column.

## Arguments
- x: A column, named bare inside a frame method or as `frame["x"]`.

## Value
An `Int`.

## Missing values
`NA` counts as one value: a variable with missing values has one distinct value more than its values present have.

## Details
Exact: the distinct keys under the identity relation of section 3.7, which `sort` and grouping use too.

## A common mistake
Calling it as a method of the frame, as `mean` is called. It takes a column, so it stands inside a definition or is given the column:

> dataset(wages).n_distinct(firm)
error[Z0401]: `n_distinct` cannot be called on a Frame
  = help: `n_distinct` takes a Column first; `x.n_distinct(...)` is `n_distinct(x, ...)`

Write `n_distinct(dataset(wages)["firm"])`, or inside a definition `dataset(wages).aggregate(firms = n_distinct(firm))`.

## Coming from R or Stata
| d.aggregate(firms = n_distinct(firm), by: year) | dplyr::summarise(dplyr::group_by(d, year), firms = dplyr::n_distinct(firm)) | egen tag = tag(year firm), then collapse (sum) firms = tag, by(year) |
| n_distinct(d["firm"]) | length(unique(d$firm)) | egen tag = tag(firm), then count if tag |

R's `unique` and dplyr's `n_distinct` count `NA` as a value, as here. Stata's `egen tag()`
leaves the observations with a missing value out unless `missing` is given.

## Related tasks
- For the number of rows, use `n`.
- For the different values themselves, use `unique`.
- For one row of each value, use `distinct`.
- For the values that repeat, use `duplicates`.
- For the count of each value, use `tabulate`.

## References
Language Reference, section 3.7 ("Identity").

## See also
n, summary

## Examples
// Of a whole column: the workers and the firms of the shipped wages.
> let wages = dataset(wages)
> n_distinct(wages["worker"])
> n_distinct(wages["firm"])
3911
397

// Inside `aggregate`: the firms and the workers seen in each year.
> wages.aggregate(firms = n_distinct(firm), workers = n_distinct(worker), by: year).head(3)
frame · 3 rows × 3 variables

      year   firms   workers
  1   2010     240       537
  2   2011     317      1108
  3   2012     353      1510
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 4)

// Inside `add`, the count of the group on each of its rows: the years each worker is seen in.
> wages.add(years = n_distinct(year), by: worker).select([worker, year, years]).head(3)
frame · 3 rows × 3 variables · panel worker × year

      worker   year   years
  1        1   2015       7
  2        1   2016       7
  3        1   2017       7

// `NA` counts as one value: 1, 2 and the missing one.
> n_distinct(frame(x = [1, 2, 2, NA, NA])["x"])
3
