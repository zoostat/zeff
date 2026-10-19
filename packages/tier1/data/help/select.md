# select

## Summary
Keep only the named variables

## Description
A frame with only the variables named, in the order named: one bare name, a list of names, or a
range `a..b` of every variable from `a` to `b` by position (section 3.7 of the Language
Reference). Nothing is copied: the new frame shares its parent's columns, and every variable keeps
its metadata. A name the frame does not have is an error with a suggestion (Z0101). Like dplyr's
`select` and Stata's `keep varlist`.

## Arguments
- data: The frame.
- vars: The variables to keep: `wage`, `[id, wage]` or `wage..hours`.

## Value
A `Frame` with the same rows and the variables named.

## See also
drop, rename, where

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.select([hours, id])
frame · 3 rows × 2 variables

      hours   id
  1      40    1
  2      35    2
  3      NA    3
> wages.select(wage..hours).variables()
wage
hours
