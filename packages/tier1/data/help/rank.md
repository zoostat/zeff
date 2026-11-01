# rank

## Summary
Rank of each value, smallest first; NA stays NA

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

## Details
Closed form: the rows are sorted once in the order of section 3.7 of the Language Reference
(numbers numerically, text by code point, dates chronologically, an ordered category by its
levels), stably, and each run of equal values is given its ranks. A category whose levels are
not ordered has no order to rank by and is refused (Z0221).

## References
R Core Team (2026), R: A Language and Environment for Statistical Computing, `rank`.

## See also
sort, bin, quantile

## Examples
> let r = frame(name = ["Ann", "Bo", "Cy", "Di", "Ed"], score = [72.5, 90, 72.5, NA, 60])
> r.add(avg = rank(score), low = rank(score, ties: min), high = rank(score, ties: max), first = rank(score, ties: first))
frame · 5 rows × 6 variables

      name   score   avg   low   high   first
  1   Ann     72.5   2.5     2      3       2
  2   Bo        90     4     4      4       4
  3   Cy      72.5   2.5     2      3       3
  4   Di        NA    NA    NA     NA      NA
  5   Ed        60     1     1      1       1
