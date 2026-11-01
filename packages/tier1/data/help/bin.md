# bin

## Summary
Category from numeric intervals, (a, b] by default as in R

## Description
Each value's interval among the breaks, as an ordered category whose levels are the intervals
in order. With `closed: right`, the default, a value `v` falls in `(a, b]` when `a < v <= b`,
as R's `cut`; with `closed: left` it falls in `[a, b)` when `a <= v < b`, as Stata's `egen cut`.
`-Inf` and `Inf` as the first and last breaks make the end intervals open, so that every value
has one. A value outside every interval becomes `NA`, with a warning giving the count. The
levels are labelled as R labels them, `(0,10]`, unless `labels:` names them.

## Arguments
- x: A variable of Ints or Floats.
- breaks: A list of at least two numbers, in any order; `-Inf` and `Inf` are open ends.
- labels: A list of texts, one per interval, in the order of the sorted breaks.
- closed: `right` for `(a, b]` or `left` for `[a, b)`; `right` unless given.

## Value
A column of an ordered category, one level per interval, `NA` where the value was `NA` or
outside every interval.

## Details
Exact: the breaks are sorted and compared as doubles, so a value equal to a break goes to the
interval whose closed end it is. A break repeated (Z0401), `NA` among the breaks, fewer than
two breaks, or `labels:` of another length than the intervals, is refused.

## See also
rank, category, recode, quantile

## Examples
> let pay = frame(worker = [1, 2, 3, 4, 5, 6], wage = [18.08, 8.5, 20.93, 15.68, 26.39, 33.07])
> pay.add(band = bin(wage, breaks: [-Inf, 10, 20, Inf]))
frame · 6 rows × 3 variables

      worker    wage   band
  1        1   18.08   (10,20]
  2        2     8.5   (-Inf,10]
  3        3   20.93   (20, Inf]
  4        4   15.68   (10,20]
  5        5   26.39   (20, Inf]
  6        6   33.07   (20, Inf]
> pay.add(band = bin(wage, breaks: [0, 10, 20], labels: ["low", "high"], closed: left))
frame · 6 rows × 3 variables

      worker    wage   band
  1        1   18.08   high
  2        2     8.5   low
  3        3   20.93   NA
  4        4   15.68   high
  5        5   26.39   NA
  6        6   33.07   NA
warning: `bin`: 3 rows outside every interval became NA (line 3)
  = help: the intervals run from 0 to 20; `-Inf` and `Inf` as the first and last breaks keep every value
> pay.add(band = bin(wage, breaks: [10, 20, 30, 40]))
frame · 6 rows × 3 variables

      worker    wage   band
  1        1   18.08   (10,20]
  2        2     8.5   NA
  3        3   20.93   (20,30]
  4        4   15.68   (10,20]
  5        5   26.39   (20,30]
  6        6   33.07   (30,40]
warning: `bin`: 1 row outside every interval became NA (line 4)
  = help: the intervals run from 10 to 40; `-Inf` and `Inf` as the first and last breaks keep every value
