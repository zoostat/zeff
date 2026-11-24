# summary

## Summary
Table of N, missing, mean, SD, min and max

## In brief
- `d.summary()`: every variable of numbers, one row each: N, missing, mean, sd, min and max.
- `d.summary([wage, hours])`: the variables named.
- `d.summary([wage], stats: [n, mean, sd])`: the statistics chosen, in the order wanted.
- `d.summary([wage], by: year)`: group by group.

## Description
One row per variable with its count of non-missing values (`N`), its count of missing values, and
its mean, sample standard deviation (n − 1), minimum and maximum, computed as `mean` and `sd`
compute them: missing values are skipped and the rows skipped are counted in one warning under the
table. Without `vars`, every Bool, Int or Float variable is summarised, in the frame's order; a
Bool variable counts `true` as 1. `stats:` chooses the statistics and their order from the six
words `n`, `missing`, `mean`, `sd`, `min` and `max`; any other word is refused.

Statistics print with three decimals, as section 10.2 of the Language Reference prints tables;
`mean` and `sd` on their own print six significant digits. A variable whose values are all missing
shows `NA` throughout, with a warning; one value alone has an `NA` standard deviation, and so has
a variable holding `Inf`.

## Arguments
- data: The frame.
- vars: The variables to summarise, in a list, as in `[wage, hours]`; one may be written alone.
  Omitted, every Bool, Int or Float variable.
- stats: The statistics to show, as words in a list, in the order wanted: `stats: [mean, sd]`.
  Omitted, all six in the order N, missing, mean, sd, min, max.

## Value
A `Table`, with the `by:` variables as its first columns when groups are asked for, which prints as shown and can be passed to `print`.

## Missing values
Each statistic is taken over the values present. The table counts the missing values of each variable in a column of its own, and one warning under it counts the rows skipped. A variable whose values are all missing shows `NA` throughout.

## Details
The mean is a compensated sum corrected once; the standard deviation is the corrected two-pass
algorithm with the deviations scaled before squaring; see `mean` and `sd`. Minimum and maximum are
taken over the non-missing values, so `Inf` and `-Inf` appear as themselves.

## A common mistake
Asking for a statistic the table does not have, such as the median. `stats:` chooses among six words:

> dataset(wages).summary([wage], stats: [median])
error[Z0402]: `median` is not one of the allowed words
  = help: did you mean `mean`?

For other statistics name them yourself: `dataset(wages).aggregate(median = median(wage), p90 = quantile(wage, p: 0.9))`.

## Coming from R or Stata
| d.summary([wage, hours]) | summary(d[, c("wage", "hours")]) | summarize wage hours |
| d.summary([wage], by: year) | tapply(d$wage, d$year, summary) | tabstat wage, by(year) statistics(n mean sd min max) |

R's `summary` gives the quartiles and the mean, no standard deviation, and the count of `NA`
only where there are some. Stata's `summarize` gives the count, the mean, the standard
deviation, the minimum and the maximum, as here, without the count of missing values.

## Related tasks
- For the type and an example of each variable, use `describe`.
- For statistics of your own choosing, group by group, use `aggregate`.
- For the quantiles of a variable, use `quantile`.
- For the count of each value of a category, use `tabulate`.
- For one statistic as a number, use `mean`.

## References
Chan, Golub and LeVeque (1983), Algorithms for computing the sample variance: analysis and
recommendations, The American Statistician 37, 242–247.

## See also
mean, sd, describe

## Examples
// Three variables of the shipped wages. The warning counts the rows skipped, variable by variable.
> let wages = dataset(wages)
> wages.summary([wage, hours, educ])
summary · wages

  variable        N   missing     mean       sd     min       max
  wage       18,201       211   27.370   14.027   3.530   152.390
  hours      18,377        35   37.945    8.158   7.000    69.200
  educ       18,412         0   13.491    2.485   8.000    20.000
warning[W0201]: 211 rows with NA in wage and 35 with NA in hours were skipped (line 2)

// Every variable of numbers when none is named.
> dataset(firms).summary()
summary · firms

  variable     N   missing      mean        sd     min       max
  firm       400         0   200.500   115.614   1.000   400.000
  size       400         0     9.777     9.677   0.000    39.000

// The statistics chosen, in the order wanted.
> wages.summary([wage, hours], stats: [mean, sd, n])
summary · wages

  variable     mean       sd        N
  wage       27.370   14.027   18,201
  hours      37.945    8.158   18,377
warning[W0201]: 211 rows with NA in wage and 35 with NA in hours were skipped (line 4)

// Group by group with `by:`.
> wages.summary([wage], by: female)
summary · by female

  female   variable       N   missing     mean       sd     min       max
  male     wage       9,375       116   29.189   14.787   4.740   152.390
  female   wage       8,826        95   25.437   12.895   3.530   140.240
warning[W0201]: 211 rows with NA in wage were skipped (line 5)

// The variables by the name of the argument, and the counts alone.
> wages.summary(vars: [wage, exper], stats: [n, missing])
summary · wages

  variable        N   missing
  wage       18,201       211
  exper      18,390        22
warning[W0201]: 211 rows with NA in wage and 22 with NA in exper were skipped (line 6)
