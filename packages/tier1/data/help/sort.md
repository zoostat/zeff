# sort

## Summary
Sort rows; stable, NA last

## Description
The rows in the order of the variables named: by the first, then by the second within equal values
of the first, and so on. The sort is stable, so rows with equal keys keep their previous order, and
`NA` comes last whether ascending or descending. Numbers sort numerically, text by code point (never
by locale, so the order is the same on every machine), dates chronologically, categories by level
position, and `false` before `true` (section 3.7 of the Language Reference, "Identity"). Nothing is
copied but the order: every variable keeps its metadata. Like dplyr's `arrange` and Stata's `sort`,
which also puts missing values last.

## Arguments
- data: The frame.
- vars: The variables to sort by, in order of precedence: `wage`, `[region, wage]` or a range.
- desc: `true` sorts every key from largest to smallest; `NA` stays last.

## Value
A `Frame` with the same variables and the rows reordered.

## See also
where, head, tail

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.sort(wage)
frame · 3 rows × 3 variables

      id   wage   hours
  1    2     19      35
  2    1   23.4      40
  3    3   31.2      NA
> wages.sort(hours, desc: true)
frame · 3 rows × 3 variables

      id   wage   hours
  1    1   23.4      40
  2    2     19      35
  3    3   31.2      NA
