# row

## Summary
Mean or sum across variables in each row

## In brief
- `row([q1, q2, q3], stat: mean)`: each row's mean of the three items, over the values present.
- `row([q1..q6], stat: sum)`: each row's total, the variables given as a range.
- `min_valid: 5` gives `NA` to a row with fewer than five of them answered.

## Description
A mean or a sum of several variables within each row, such as a scale score from its items.
For each row the values present among the variables named are averaged or added; `stat:` says
which, and has no default. A row with fewer than `min_valid:` values present is `NA`.

It stands inside `add` or `update`, where the frame's variables are in scope, and takes them as
a list, `[q1, q2, q3]`, or a range by position, `[eng1..eng6]`. The variables must be Ints or
Floats; a Bool is refused with the conversion that turns it into 0 and 1.

## Arguments
- vars: The variables to combine: a list of names, or a range by position.
- stat: `mean` or `sum`; always given.
- min_valid: The number of values a row needs for a result; 1 unless given.

## Value
A column of Floats, one value per row.

## Missing values
An `NA` among a row's values is left out: the mean is over the values present and the sum adds
them, as R's `rowMeans` and `rowSums` with `na.rm = TRUE`. A row with fewer than `min_valid:`
values present is `NA`, so with the default a row with no value at all is `NA`, not 0. Nothing
is printed about the rows; `n_missing` counts the `NA` in the result.

## Details
### The rule for a scale with missing items
`min_valid:` is the rule psychometrics applies to a scale score: so many of the items
answered, or no score. R's and Stata's functions leave that rule to the user. It applies to
the sum as well, though a sum over fewer items is not comparable with one over all of them; a
mean is the usual score when items may be missing.

### A sum of nothing
Stata's `egen rowtotal()` gives 0 for a row with no value; here it is `NA`, since no value was
added. `fill_missing(row(vars, stat: sum), 0)` gives Stata's answer.

### Exactness
Closed form: the compensated summation every mean and sum here uses, so the result does not
depend on the order of the variables to the last bit that a plain `+` would lose. Int
variables are read exactly as Floats below 2^53.

## A common mistake
Leaving `stat:` out. There is no default, so that a script says whether a score is a mean or
a total:

> frame(q1 = [5, 4], q2 = [3, NA]).add(score = row([q1, q2]))
error[Z0406]: `row` needs `stat:`
  = help: usage: `row([q1, q2, q3], stat: mean)` for the mean across the variables, or `stat: sum` for their sum

Write `row([q1, q2], stat: mean)`.

## Coming from R or Stata
| row([a, b, c], stat: mean) | rowMeans(d[c("a", "b", "c")], na.rm = TRUE) | egen m = rowmean(a b c) |
| row([a, b, c], stat: sum) | rowSums(d[c("a", "b", "c")], na.rm = TRUE) | egen s = rowtotal(a b c), missing |
| row([a..c], stat: mean, min_valid: 2) | none built in | none built in |

`rowSums(na.rm = TRUE)` and `rowtotal()` without `missing` give 0 for a row with no value.
`row_mean(vars)` and `row_sum(vars)` of releases 0.5.0 to 0.7.0 are `row(vars, stat: mean)`
and `row(vars, stat: sum)`.

## Related tasks
- For the mean of one variable down the rows, use `mean`.
- To count the values missing in a column, use `n_missing`.
- To put a value where the result is missing, use `fill_missing`.
- To put the items on one scale before combining them, use `standardize`.
- To compute anything else across variables, row by row, write the expression in `add`.

## References
R Core Team (2026), R: A Language and Environment for Statistical Computing, `rowMeans` and
`rowSums`. Stata, `egen` with `rowmean()` and `rowtotal()`.

## See also
mean, sum, n_missing, fill_missing, standardize

## Examples
// A scale score: the mean of the items answered in each row.
> let d = frame(q1 = [5, 4, NA], q2 = [3, NA, NA], q3 = [4, 4, 2])
> d.add(score = row([q1, q2, q3], stat: mean))
frame · 3 rows × 4 variables

      q1   q2   q3   score
  1    5    3    4       4
  2    4   NA    4       4
  3   NA   NA    2       2

// The variables as a range, and no score for a row with fewer than two answers.
> d.add(score = row([q1..q3], stat: mean, min_valid: 2))
frame · 3 rows × 4 variables

      q1   q2   q3   score
  1    5    3    4       4
  2    4   NA    4       4
  3   NA   NA    2      NA

// The total across the variables; a row with no value is `NA`, not 0.
> let e = frame(q1 = [5, 4, NA], q2 = [3, NA, NA], q3 = [4, 4, NA])
> e.add(total = row([q1, q2, q3], stat: sum))
frame · 3 rows × 4 variables

      q1   q2   q3   total
  1    5    3    4      12
  2    4   NA    4       8
  3   NA   NA   NA      NA
