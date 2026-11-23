# aggregate

## Summary
One row per group with reducers such as mean(x), sd(x), n() or n_distinct(x)

## In brief
- `d.aggregate(m = mean(wage), by: year)`: one row for each year, with the mean wage of the year.
- `d.aggregate(m = mean(wage), s = sd(wage), rows = n(), by: [year, industry])`: several statistics, groups of two variables.
- `d.aggregate(m = mean(wage))`: without `by:`, one row for the whole frame.

## Description
A new frame with one row per group of the `by:` variables and one column per definition, each
definition a statistic of the group: a reducer of the data package (`mean`, `median`, `sd`,
`variance`, `sum`, `min`, `max`, `n`, `n_distinct`, `quantile`, `iqr`, `mad`, `skewness`,
`kurtosis`) or arithmetic over reducers, `sd(wage) / sqrt(n())`. The definitions are
evaluated left to right and a later one may use an earlier one by name. The groups come in key
order, `NA` last and forming its own group; the `by:` variables are the first columns of the
result and keep their labels, while a definition starts with no metadata. Without `by:` the
result has one row. A definition that gives a value per row rather than one value per group is
refused (Z0401): `add` is for those. The panel declaration is dropped, with a note, since the
rows are summarised.

## Arguments
- data: The frame.
- ...: The definitions, `name = expr`, each a statistic of the group: a reducer or arithmetic over reducers.
- by: The variable, or list of variables, that defines the groups; one group without it.

## Value
A frame: the `by:` variables, then the definitions, one row per group.

## Missing values
Each reducer skips the `NA` of its variable, and one warning counts the rows skipped over all the groups; a group whose values are all `NA` has `NA`. `n()` counts every row of the group. A `by:` variable that is `NA` forms a group of its own, the last.

## Details
Exact: the rows are grouped by the kernel's identity on the key values (section 3.7 of the
Language Reference), each definition is evaluated on each group's rows as the reducer defines it,
and the values are placed in key order; nothing else is computed. The reducers' warnings for the
rows with `NA` skipped are counted once over the groups, as `summary` counts them.
A definition that takes the name of a variable of the frame, `spend = mean(spend)`, is that
variable for the definitions after it, one value for each group: `sd(spend)` written after it is
the spread of that single value, 0. A later definition that reads such a name is told so in a
note, which says to define it first or to give the summary a name of its own,
`mean_spend = mean(spend)`.

## A common mistake
Writing a definition that has a value for each row. A definition here is one value for each group:

> dataset(wages).aggregate(double = wage * 2, by: industry)
error[Z0401]: `double` of `aggregate` gives a Column, not one value per group
  = help: a definition reduces the group to one value, as `mean(wage)` or `sd(wage) / sqrt(n())` do; `add` keeps a value per row

Reduce the group, `aggregate(double = mean(wage) * 2, by: industry)`; for a value in each row, use `add`.

## Coming from R or Stata
| d.aggregate(m = mean(wage), by: year) | aggregate(wage ~ year, d, mean), or dplyr::summarise(dplyr::group_by(d, year), m = mean(wage, na.rm = TRUE)) | collapse (mean) m = wage, by(year) |
| d.aggregate(m = mean(wage), rows = n(), by: year) | dplyr::summarise(dplyr::group_by(d, year), m = mean(wage, na.rm = TRUE), rows = dplyr::n()) | collapse (mean) m = wage (count) rows = wage, by(year) |

In dplyr a mean over a group with a missing value is `NA` unless `na.rm = TRUE`; here the
reducers skip `NA` and a warning counts the rows skipped. Stata's `collapse` skips them and
says nothing, its `(count)` counts the values present where `n()` counts the rows, and it
replaces the dataset in memory; `aggregate` gives a new frame.

## Related tasks
- For a statistic of each group written on every row of the group, use `add` with `by:`.
- For the usual statistics of every variable, use `summary`.
- For the count of each value of a variable, use `tabulate`.
- To bring the summary back beside the rows, use `merge`.

## See also
add, summary, mean, n, n_distinct, distinct

## Examples
// One row for each year of the shipped wages. The warning counts the rows whose wage is missing.
> let wages = dataset(wages)
> wages.aggregate(mean_wage = mean(wage), workers = n_distinct(worker), rows = n(), by: year).head(4)
frame · 4 rows × 4 variables

      year   mean_wage   workers   rows
  1   2010     22.4203       537    537
  2   2011     23.5616      1108   1108
  3   2012     24.2861      1510   1510
  4   2013     24.8461      1839   1839
warning[W0201]: 211 rows with NA in wage were skipped (line 2)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 2)

// Arithmetic over reducers, and a later definition that uses an earlier one.
> wages.aggregate(avg = mean(wage), spread = sd(wage), se = spread / sqrt(n() - n_missing(wage)), by: female)
frame · 2 rows × 4 variables

      female       avg    spread         se
  1   male     29.1889   14.7867   0.152716
  2   female    25.437   12.8947   0.137255
warning[W0201]: 211 rows with NA in wage were skipped (line 3)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 3)

// Groups of two variables.
> wages.aggregate(avg = mean(wage), by: [female, union])
frame · 4 rows × 3 variables

      female   union       avg
  1   male     no      28.1946
  2   male     yes     31.3718
  3   female   no      24.5954
  4   female   yes      27.198
warning[W0201]: 211 rows with NA in wage were skipped (line 4)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 4)

// Without `by:`, one row.
> wages.aggregate(total_hours = sum(hours), rows = n())
frame · 1 row × 2 variables

      total_hours    rows
  1        697309   18412
warning[W0201]: 35 rows with NA in hours were skipped (line 5)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 5)
