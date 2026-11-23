# drop

## Summary
Remove the named variables

## Description
A frame without the variables named: one bare name, a list of names, or a range `a..b` by
position. Nothing is copied: the new frame shares its parent's columns, and every remaining
variable keeps its metadata. A name the frame does not have is an error with a suggestion (Z0101).

## Arguments
- data: The frame.
- vars: The variables to remove: `hours`, `[id, hours]` or `wage..hours`.

## Value
A `Frame` with the same rows and the variables that were not named.

## Missing values
A variable is removed whatever it holds; no row is dropped. To remove rows with missing values, use `drop_missing`.

## A common mistake
Misspelling a name. Nothing is removed, and the message offers the nearest name:

> dataset(firms).drop(sise)
error[Z0101]: no variable `sise` in `firms`
  = help: did you mean `size`? If you meant a category level, write it in quotes: `"size"`

Write `dataset(firms).drop(size)`.

## Coming from R or Stata
| d.drop(hours) | d$hours <- NULL, or dplyr::select(d, -hours) | drop hours |
| d.drop([id, hours]) | dplyr::select(d, -c(id, hours)) | drop id hours |

Stata's `drop if` removes rows; here rows are chosen with `where`, and `drop` is for variables
alone. R's `d$hours <- NULL` and Stata's `drop` change the data they are given; `drop` gives a
new frame.

## Related tasks
- To name the variables to keep instead, use `select`.
- To remove rows, keep the others with `where`.
- To remove the rows with a missing value, use `drop_missing`.

## See also
select, rename, where

## Examples
// One variable removed.
> let firms = dataset(firms)
> firms.drop(founded).head(3)
frame · 3 rows × 3 variables

      firm   industry      size
  1      1   Agriculture      3
  2      2   Agriculture      5
  3      3   Agriculture      3

// Several, in a list.
> firms.drop([industry, founded]).variables()
firm
size

// A range, by position.
> firms.drop(industry..founded).variables()
firm
