# drop

## Summary
Remove the named variables

## Description
A frame without the variables named: one bare name, a list of names, or a range `a..b` by
position. Nothing is copied: the new frame shares its parent's columns, and every remaining
variable keeps its metadata. A name the frame does not have is an error with a suggestion (Z0101).
Like Stata's `drop varlist` and dplyr's `select(-x)`.

## Arguments
- data: The frame.
- vars: The variables to remove: `hours`, `[id, hours]` or `wage..hours`.

## Value
A `Frame` with the same rows and the variables that were not named.

## See also
select, rename, where

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.drop(hours)
frame · 3 rows × 2 variables

      id   wage
  1    1   23.4
  2    2     19
  3    3   31.2
