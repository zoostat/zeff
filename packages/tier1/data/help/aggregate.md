# aggregate

## Summary
One row per group with reducers such as mean(x), sd(x), n() or n_distinct(x)

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
rows are summarised. Like dplyr's `summarise` and Stata's `collapse`.

## Arguments
- data: The frame.
- ...: The definitions, `name = expr`, each a statistic of the group: a reducer or arithmetic over reducers.
- by: The variable, or list of variables, that defines the groups; one group without it.

## Value
A frame: the `by:` variables, then the definitions, one row per group.

## Details
Exact: the rows are grouped by the kernel's identity on the key values (section 3.7 of the
Language Reference), each definition is evaluated on each group's rows as the reducer defines it,
and the values are placed in key order; nothing else is computed. The reducers' warnings for the
rows with `NA` skipped are counted once over the groups, as `summary` counts them.

## See also
add, summary, mean, n, n_distinct, distinct

## Examples
> let sales = frame(firm = [2, 1, 2, 1, 3, 1], year = [2021, 2020, 2020, 2020, 2021, 2021], revenue = [12.5, 8, 11, 9, NA, 10])
> sales.aggregate(total = sum(revenue), firms = n_distinct(firm), rows = n(), by: year)
frame · 2 rows × 4 variables

      year   total   firms   rows
  1   2020      28       2      3
  2   2021    22.5       3      3
warning[W0201]: 1 row with NA in revenue was skipped (line 2)
> sales.aggregate(avg = mean(revenue), spread = sd(revenue), se = spread / sqrt(n()), by: firm)
frame · 3 rows × 4 variables

      firm     avg    spread        se
  1      1       9         1   0.57735
  2      2   11.75   1.06066      0.75
  3      3      NA        NA        NA
warning[W0201]: all 1 rows have NA in revenue: the result is NA (line 3)
> sales.aggregate(total = sum(revenue), rows = n())
frame · 1 row × 2 variables

      total   rows
  1    50.5      6
warning[W0201]: 1 row with NA in revenue was skipped (line 4)
