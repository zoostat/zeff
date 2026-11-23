# rank

## Summary
Rank of each value, smallest first; NA stays NA

## In brief
- `d.add(r = rank(wage))`: 1 for the smallest; equal values share the mean of their positions.
- `rank(wage, ties: min)`: equal values share the first of their positions; `max` the last; `first` numbers them in row order.
- `d.add(r = rank(wage), by: year)`: the ranks start again in each group.
- `rank(-wage)`: 1 for the largest.

## Description
The position of each value when the column is sorted, 1 for the smallest. Equal values are
ties, and `ties:` says what they receive: `average` (the default) gives each the mean of the
positions they span, as R's `rank` and Stata's `egen rank` do; `min` gives each the first of
those positions, `max` the last, and `first` numbers them in row order so that every rank is
distinct. An `NA` has no rank and stays `NA`, as R's `na.last = "keep"`; the values present
are ranked among themselves. Under `by:` the ranks restart in each group.

## Arguments
- x: A variable of numbers, dates, text, Bools or an ordered category.
- ties: `average`, `min`, `max` or `first`; `average` unless given.

## Value
A column of Floats under `ties: average`, since a tied rank may end in a half, and of Ints under
the other three.

## Missing values
`NA` has no rank and stays `NA`; the values present are ranked among themselves.

## Details
Closed form: the rows are sorted once in the order of section 3.7 of the Language Reference
(numbers numerically, text by code point, dates chronologically, an ordered category by its
levels), stably, and each run of equal values is given its ranks. A category whose levels are
not ordered has no order to rank by and is refused (Z0221).

## A common mistake
Ranking a category whose levels have no order. There is nothing to say which level comes first:

> dataset(firms).add(r = rank(industry))
error[Z0221]: `rank` orders its values, but the levels of this category are not ordered
  = help: `category(x, levels: [...], ordered: true)` gives the levels an order (section 4.2 of the Language Reference)

Give the levels an order with `category(x, levels: [...], ordered: true)`, or rank a number.

## Coming from R or Stata
| d.add(r = rank(wage)) | d$r <- rank(d$wage, na.last = "keep") | egen r = rank(wage) |
| d.add(r = rank(wage, ties: min)) | rank(d$wage, ties.method = "min", na.last = "keep") | egen r = rank(wage), track |
| d.add(r = rank(-wage)) | rank(-d$wage, na.last = "keep") | egen r = rank(-wage) |
| d.add(r = rank(wage), by: year) | ave(d$wage, d$year, FUN = function(x) rank(x, na.last = "keep")) | bysort year: egen r = rank(wage) |

R's `rank` puts `NA` last and gives it a rank unless `na.last = "keep"`; here, as in Stata,
a missing value has no rank.

## Related tasks
- To put the rows themselves in order, use `sort`.
- To number the rows as they stand, use `row_number`.
- To cut a variable into groups, use `bin`.
- For the value at a given share of the ranks, use `quantile`.

## References
R Core Team (2026), R: A Language and Environment for Statistical Computing, `rank`.

## See also
sort, bin, quantile

## Examples
// The rank of each wage of one year, 1 for the smallest; shown from the top.
> let w = dataset(wages).where(year == 2019).select([worker, year, wage, educ])
> w.add(r = rank(wage)).sort(wage, desc: true).head(3)
frame · 3 rows × 5 variables · panel worker × year

      worker   year     wage   educ      r
  1     1651   2019   104.59     18   1492
  2      997   2019    95.49     15   1491
  3     3540   2019    93.81     13   1490

// The four ways of ranking equal values, on years of schooling, where many are equal.
> w.add(avg = rank(educ), low = rank(educ, ties: min), high = rank(educ, ties: max), first = rank(educ, ties: first)).head(4)
frame · 4 rows × 8 variables · panel worker × year

      worker   year    wage   educ      avg    low   high   first
  1        1   2019   26.39     17   1377.5   1336   1419    1336
  2        7   2019   23.39     13    652.5    552    753     552
  3       13   2019   22.98     10      138     96    180      96
  4       15   2019   20.14     17   1377.5   1336   1419    1337

// 1 for the largest: the rank of the negative.
> w.add(r = rank(-wage)).sort(r).head(3)
frame · 3 rows × 5 variables · panel worker × year

      worker   year     wage   educ   r
  1     1651   2019   104.59     18   1
  2      997   2019    95.49     15   2
  3     3540   2019    93.81     13   3

// Within each group with `by:`: the place of a worker's wage among the wages of the same year.
> dataset(wages).add(r = rank(wage, ties: min), by: year).where(worker == 1).select([worker, year, wage, r]).head(3)
frame · 3 rows × 4 variables · panel worker × year

      worker   year    wage     r
  1        1   2015   18.08   570
  2        1   2016   17.28   486
  3        1   2017   20.93   695
