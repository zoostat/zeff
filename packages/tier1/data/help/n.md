# n

## Summary
Row count, as a reducer inside add and aggregate

## In brief
- `d.add(share = hours / n())`: the number of rows of the frame, on every row.
- `d.add(size = n(), by: firm)`: the number of rows of each group, on every row of the group.
- `d.aggregate(rows = n(), by: year)`: one row for each group, with its count.

## Description
The number of rows of the frame a definition is being evaluated for: inside `add` or `update`, the
whole frame, or the group under `by:`. It is the reducer that turns a count into a share,
`add(share = x / n())`. Outside a frame method there is no frame
to count, and `n()` is an error that points to `n_rows`.

## Arguments

## Value
An `Int`.

## Missing values
Every row is counted, whatever it holds. To count the values of a variable that are present, take its missing values from the count: `n() - n_missing(wage)`.

## Details
Exact: the number of rows of the frame, or of the group, that the definition is evaluated for.

## A common mistake
Calling it where there is no frame. The rows it counts are those of the frame a definition is evaluated for:

> n()
error[Z0401]: `n()` counts the rows of a frame and stands inside `add` or `aggregate`
  = help: write `wages.n_rows()` for a frame's row count, or `wages.add(share = x / n())` for a count inside a definition

For the rows of a frame write `d.n_rows()`; inside `add` or `aggregate` write `n()`.

## Coming from R or Stata
| d.add(rows = n(), by: year) | dplyr::add_count(d, year, name = "rows") | bysort year: generate rows = _N |
| d.aggregate(rows = n(), by: year) | dplyr::count(d, year, name = "rows") | contract year, freq(rows) |
| d.add(share = hours / n()) | dplyr::mutate(d, share = hours / dplyr::n()) | generate share = hours / _N |

Stata's `_N` is the number of observations of the dataset, or of the group under `by`; `n()` is
the same count, of the frame or of the group under `by:`.

## Related tasks
- For the number of rows of a frame as a value, use `n_rows`.
- For the number of different values of a variable, use `n_distinct`.
- For the position of each row, use `row_number`.
- For the missing values of a variable, use `n_missing`.
- For the count of each value of a variable, use `tabulate`.

## References
Language Reference, section 6 ("Statistics as methods and reducers").

## See also
n_rows, n_distinct, add

## Examples
// The number of rows of each group, on every row of the group: how many rows each worker has.
> let wages = dataset(wages)
> wages.add(rows = n(), by: worker).select([worker, year, rows]).head(3)
frame · 3 rows × 3 variables · panel worker × year

      worker   year   rows
  1        1   2015      7
  2        1   2016      7
  3        1   2017      7

// One row for each group, with its count.
> wages.aggregate(rows = n(), by: year).head(3)
frame · 3 rows × 2 variables

      year   rows
  1   2010    537
  2   2011   1108
  3   2012   1510
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 3)

// In arithmetic: the standard error of a mean, the count being of the values present.
> wages.aggregate(mean = mean(wage), se = sd(wage) / sqrt(n() - n_missing(wage)), by: female)
frame · 2 rows × 3 variables

      female      mean         se
  1   male     29.1889   0.152716
  2   female    25.437   0.137255
warning[W0201]: 211 rows with NA in wage were skipped (line 4)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 4)

// The values present are the rows less the missing values.
> wages.aggregate(rows = n(), with_wage = n() - n_missing(wage))
frame · 1 row × 2 variables

       rows   with_wage
  1   18412       18201
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 5)
