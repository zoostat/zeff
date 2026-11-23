# row

## Summary
Mean, sum or count of missing values across variables in each row

## In brief
- `row([q1, q2, q3], stat: mean)`: each row's mean of the three items, over the values present.
- `row([q1..q6], stat: sum)`: each row's total, the variables given as a range.
- `min_valid: 5` gives `NA` to a row with fewer than five of them answered.
- `row([q1..q6], stat: n_missing)`: how many of the six are missing in each row.

## Description
A mean or a sum of several variables within each row, such as a scale score from its items.
For each row the values present among the variables named are averaged or added; `stat:` says
which, and has no default. A row with fewer than `min_valid:` values present is `NA`.

`stat: n_missing` counts instead: the number of the variables named whose value is missing in
the row, from 0 to the number of variables. It is the count behind "how complete is this
record": `where(row([q1..q6], stat: n_missing) == 0)` keeps the rows with every item answered,
and the count itself can be tabulated or used as a rule of its own.

It stands inside `add`, `update` or `where`, where the frame's variables are in scope, and takes
them as a list, `[q1, q2, q3]`, or a range by position, `[eng1..eng6]`. For a mean or a sum the
variables must be Ints or Floats; a Bool is refused with the conversion that turns it into 0 and
1. A count of missing values takes variables of any type, text, dates and categories among
them.

## Arguments
- vars: The variables to combine: a list of names, or a range by position.
- stat: `mean`, `sum` or `n_missing`; always given.
- min_valid: The number of values a row needs for a mean or a sum; 1 unless given. A count of
  missing values needs none, and `min_valid:` written with `stat: n_missing` changes nothing,
  which a note says (W0401).

## Value
A column of Floats, one value per row, for `mean` and `sum`; a column of Ints for `n_missing`,
with a number in every row.

## Missing values
An `NA` among a row's values is left out: the mean is over the values present and the sum adds
them, as R's `rowMeans` and `rowSums` with `na.rm = TRUE`. A row with fewer than `min_valid:`
values present is `NA`, so with the default a row with no value at all is `NA`, not 0. Nothing
is printed about the rows; `n_missing` counts the `NA` in the result. Under
`stat: n_missing` the missing values are what is counted: the result is never `NA`, and a row
with every variable missing has the number of variables. An empty text is a value, not a
missing one.

## Details
### The rule for a scale with missing items
`min_valid:` is the rule psychometrics applies to a scale score: so many of the items
answered, or no score. R's and Stata's functions leave that rule to the user. It applies to
the sum as well, though a sum over fewer items is not comparable with one over all of them; a
mean is the usual score when items may be missing.

### A sum of nothing
Stata's `egen rowtotal()` gives 0 for a row with no value; here it is `NA`, since no value was
added. `fill_missing(row(vars, stat: sum), 0)` gives Stata's answer.

### Counting missing values
`stat: n_missing` looks at each variable's value in the row and counts those that are `NA`,
whatever the variable's type. A variable named twice in the list is one variable. The number
of values present is the number of variables less the count:
`3 - row([q1, q2, q3], stat: n_missing)`.

### Exactness
Closed form: the compensated summation every mean and sum here uses, so the result does not
depend on the order of the variables to the last bit that a plain `+` would lose. Int
variables are read exactly as Floats below 2^53.

## A common mistake
Leaving `stat:` out. There is no default, so that a script says whether a score is a mean, a
total or a count of gaps:

> frame(q1 = [5, 4], q2 = [3, NA]).add(score = row([q1, q2]))
error[Z0406]: `row` needs `stat:`
  = help: usage: `row([q1, q2, q3], stat: mean)` for the mean across the variables, `stat: sum` for their sum, or `stat: n_missing` for the number of them that are missing

Write `row([q1, q2], stat: mean)`.

## Coming from R or Stata
| row([a, b, c], stat: mean) | rowMeans(d[c("a", "b", "c")], na.rm = TRUE) | egen m = rowmean(a b c) |
| row([a, b, c], stat: sum) | rowSums(d[c("a", "b", "c")], na.rm = TRUE) | egen s = rowtotal(a b c), missing |
| row([a..c], stat: mean, min_valid: 2) | none built in | none built in |
| row([a, b, c], stat: n_missing) | rowSums(is.na(d[c("a", "b", "c")])) | egen k = rowmiss(a b c) |

`rowSums(na.rm = TRUE)` and `rowtotal()` without `missing` give 0 for a row with no value.
Stata's `egen rownonmiss()`, the number of values present, is the number of variables less
`row(vars, stat: n_missing)`; Stata counts an empty text as missing, and here, as in R, it is
a value.
`row_mean(vars)` and `row_sum(vars)` of releases 0.5.0 to 0.7.0 are `row(vars, stat: mean)`
and `row(vars, stat: sum)`.

## Related tasks
- For the mean of one variable down the rows, use `mean`.
- To count the values missing in a column, down its rows, use `n_missing`.
- To drop the rows with a value missing among some variables, use `drop_missing`.
- To put a value where the result is missing, use `fill_missing`.
- To put the items on one scale before combining them, use `standardize`.
- To compute anything else across variables, row by row, write the expression in `add`.

## References
R Core Team (2026), R: A Language and Environment for Statistical Computing, `rowMeans` and
`rowSums`. Stata, `egen` with `rowmean()`, `rowtotal()` and `rowmiss()`.

## See also
mean, sum, n_missing, drop_missing, fill_missing, standardize

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

// How many of the items are missing in each row.
> d.add(gaps = row([q1, q2, q3], stat: n_missing))
frame · 3 rows × 4 variables

      q1   q2   q3   gaps
  1    5    3    4      0
  2    4   NA    4      1
  3   NA   NA    2      2

// The rows with no item missing.
> d.where(row([q1..q3], stat: n_missing) == 0)
frame · 1 row × 3 variables

      q1   q2   q3
  1    5    3    4

// Variables of any type are counted: a number, a text and a date.
> let people = frame(age = [34, NA, 51], name = ["Ann", "Bo", NA], seen = [date("2026-03-01"), NA, NA])
> people.add(gaps = row([age, name, seen], stat: n_missing))
frame · 3 rows × 4 variables

      age   name   seen         gaps
  1    34   Ann    2026-03-01      0
  2    NA   Bo     NA              2
  3    51   NA     NA              2
