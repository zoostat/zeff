# bin

## Summary
Ordered category from a number, by quantile groups or by intervals

## In brief
- `bin(wage, n: 4)`: the quartile groups, levels `1` to `4`; `n: 10` gives deciles.
- `bin(age, breaks: [0, 18, 65, Inf])`: the intervals `(0,18]`, `(18,65]` and `(65, Inf]`.
- `labels: ["child", "adult", "senior"]` names the levels; `to(g, Int)` gives their numbers.

## Description
Cuts a column of numbers into an ordered category, in one of two ways. Exactly one of `n:` and
`breaks:` is given.

With `n:`, the cut points are taken from the data: the sample quantiles at 1/n, 2/n and so on,
so that each of the `n` groups holds about the same number of rows. The levels are `1` to `n`
from the lowest values to the highest.

With `breaks:`, the cut points are the numbers you list, and the levels are the intervals
between neighbouring breaks, labelled as R labels them, `(0,10]`. `-Inf` and `Inf` as the first
and last breaks make the end intervals open, so that every value has one.

Either way a value equal to a cut point goes to the lower side under `closed: right`, the
default, and to the upper side under `closed: left`.

## Arguments
- x: A variable of Ints or Floats.
- n: The number of quantile groups, a whole number of at least 1; given in the place of `breaks:`.
- breaks: A list of at least two numbers, in any order; `-Inf` and `Inf` are open ends; given in the place of `n:`.
- labels: A list of texts, one per level, from the lowest level to the highest.
- closed: `right` for `(a, b]` or `left` for `[a, b)`; `right` unless given.
- type: The quantile definition used with `n:`, 1 to 9; 7 unless given, as in R.

## Value
A column of an ordered category, one row per row of `x`. Its levels are `1` to `n`, or the
intervals, or the labels given; a level with no row is still a level.

## Missing values
`NA` in `x` gives `NA`, without a note, and the quantiles of `n:` are taken over the values
present. With `breaks:`, a value outside every interval becomes `NA` with a warning (W0210)
that counts the rows. With `n:`, a column holding both `-Inf` and `Inf` can put a cut point
between them, which has no value: every group is then `NA`, with a warning (W0205).

## Details
### Quantile groups
The cut points are the sample quantiles at i/n for i from 1 to n − 1, the same numbers
`quantile` gives under the same `type:`. A row's group is one more than the number of cut
points below its value. This is R's `cut(x, quantile(x, probs), include.lowest = TRUE)`.

### Ties and empty groups
Tied values always share a group, so the groups are not always of equal size. When the column
has fewer distinct values than groups, two cut points coincide and the groups between them are
empty; the numbering is kept, as Stata's `xtile` keeps it, where R's `cut` refuses repeated
breaks. `dplyr::ntile` is a different rule: it splits the rows by rank into groups of equal
size, so tied values can land in different groups.

### Which end is closed
With `closed: right` a value `v` falls in `(a, b]` when `a < v <= b`; with `closed: left` in
`[a, b)` when `a <= v < b`. With `breaks:` the lowest break itself is therefore outside under
`closed: right`, and the highest under `closed: left`. With `n:` no value is ever outside.

### Within groups
Under `by:` the quantiles of `n:` are taken within each group of rows, so `bin(wage, n: 4)` by
year gives each worker's quartile among the workers of that year.

### Exactness
With `breaks:` the result is exact: the breaks are sorted and compared as doubles. With `n:`
the cut points carry the rounding of the quantile's interpolation, exactly as `quantile` gives
them, and the comparison with them is exact.

## A common mistake
Computing with the groups as if they were numbers. `bin` gives a category, whose levels are
names even when they read `1` to `4`:

> frame(wage = [8.5, 18.08, 26.39]).add(double = bin(wage, n: 3) * 2)
error[Z0221]: `*` does not apply to a category
  = help: a category holds levels, not numbers: compare it with `==` or `!=`, or convert it with `to(x, Int)` to its level positions

Write `to(bin(wage, n: 3), Int) * 2`.

## Coming from R or Stata
| bin(x, n: 4) | cut(x, quantile(x, 0:4 / 4), include.lowest = TRUE) | xtile g = x, nq(4) |
| to(bin(x, n: 4), Int) | findInterval(x, quantile(x, 1:3 / 4), left.open = TRUE) + 1 | xtile g = x, nq(4) |
| bin(x, breaks: [0, 10, 20]) | cut(x, c(0, 10, 20)) | none built in |
| bin(x, breaks: [0, 10, 20], closed: left) | cut(x, c(0, 10, 20), right = FALSE) | egen g = cut(x), at(0 10 20) |

Stata's `xtile` takes its cut points from `pctile`, whose rule is definition 2 of the nine, so
`bin(x, n: 4, type: 2)` gives its groups. `quantile_group(x, n: 4)` of releases 0.6.0 and
0.7.0 is `to(bin(x, n: 4), Int)`.

## Related tasks
- To see the cut points themselves, use `quantile`.
- To number the rows from lowest to highest, ties and all, use `rank`.
- To pull extreme values in to a quantile and keep the numbers, use `winsorize`.
- To count the rows in each level, use `tabulate`.
- To merge or rename levels afterwards, use `recode`.
- To turn the levels into their positions 1, 2, 3, use `to`.

## References
Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American
Statistician 50(4), 361–365.

## See also
quantile, rank, winsorize, category, recode, tabulate

## Examples
// Quartile groups: the cut points are the quartiles of the wages present.
> let pay = frame(worker = [1, 2, 3, 4, 5, 6, 7], wage = [18.08, 8.5, 20.93, 15.68, 26.39, 33.07, NA])
> pay.add(quartile = bin(wage, n: 4))
frame · 7 rows × 3 variables

      worker    wage   quartile
  1        1   18.08   2
  2        2     8.5   1
  3        3   20.93   3
  4        4   15.68   1
  5        5   26.39   4
  6        6   33.07   4
  7        7      NA   NA

// Two groups at the median, with names for the levels.
> pay.add(half = bin(wage, n: 2, labels: ["lower", "upper"]))
frame · 7 rows × 3 variables

      worker    wage   half
  1        1   18.08   lower
  2        2     8.5   lower
  3        3   20.93   upper
  4        4   15.68   lower
  5        5   26.39   upper
  6        6   33.07   upper
  7        7      NA   NA

// Intervals at numbers you choose; `-Inf` and `Inf` keep every value.
> pay.add(band = bin(wage, breaks: [-Inf, 10, 20, Inf]))
frame · 7 rows × 3 variables

      worker    wage   band
  1        1   18.08   (10,20]
  2        2     8.5   (-Inf,10]
  3        3   20.93   (20, Inf]
  4        4   15.68   (10,20]
  5        5   26.39   (20, Inf]
  6        6   33.07   (20, Inf]
  7        7      NA   NA

// Closed on the left, with labels; a wage outside every interval is `NA`, with a warning.
> pay.add(band = bin(wage, breaks: [0, 10, 20], labels: ["low", "high"], closed: left))
frame · 7 rows × 3 variables

      worker    wage   band
  1        1   18.08   high
  2        2     8.5   low
  3        3   20.93   NA
  4        4   15.68   high
  5        5   26.39   NA
  6        6   33.07   NA
  7        7      NA   NA
warning[W0210]: `bin`: 3 rows outside every interval became NA (line 5)
  = help: the intervals run from 0 to 20; `-Inf` and `Inf` as the first and last breaks keep every value

// Thirds under another quantile definition, and the group as a number.
> pay.add(third = bin(wage, n: 3, type: 6), number = to(bin(wage, n: 3, type: 6), Int))
frame · 7 rows × 4 variables

      worker    wage   third   number
  1        1   18.08   2            2
  2        2     8.5   1            1
  3        3   20.93   2            2
  4        4   15.68   1            1
  5        5   26.39   3            3
  6        6   33.07   3            3
  7        7      NA   NA          NA

// A value equal to a cut point: 20 is the median, and `closed:` says which side takes it.
> frame(score = [10, 20, 30]).add(right = bin(score, n: 2), left = bin(score, n: 2, closed: left))
frame · 3 rows × 3 variables

      score   right   left
  1      10   1       1
  2      20   1       2
  3      30   2       2

// Tied values share a group, so groups 2 and 3 are empty here; the numbering is kept.
> frame(score = [1, 1, 1, 1, 2]).add(q = bin(score, n: 4)).tabulate(q)
tabulate · the frame · q

  q       N       %
  1       4    80.0
  4       1    20.0
  ─────────────────
  Total   5   100.0
