# select

## Summary
Keep only the named variables

## Description
A frame with only the variables named, in the order named: one bare name, a list of names, or a
range `a..b` of every variable from `a` to `b` by position (section 3.7 of the Language
Reference). Nothing is copied: the new frame shares its parent's columns, and every variable keeps
its metadata. A name the frame does not have is an error with a suggestion (Z0101).

## Arguments
- data: The frame.
- vars: The variables to keep: `wage`, `[id, wage]` or `wage..hours`.

## Value
A `Frame` with the same rows and the variables named.

## Missing values
A variable is kept whatever it holds; no row is dropped.

## A common mistake
Writing a name in quotation marks, as R's `d[, "size"]` does. A variable is named bare, and text in quotation marks is text:

> dataset(firms).select("size")
error[Z0401]: `vars` of `select` must be a Vars, but this is a Str
  = help: name the variables in a list, as in `[lwage, educ]`

Write `dataset(firms).select(size)`, or `select([firm, size])` for several.

## Coming from R or Stata
| d.select([id, wage]) | d[, c("id", "wage")], or dplyr::select(d, id, wage) | keep id wage |
| d.select(wage..hours) | dplyr::select(d, wage:hours) | keep wage-hours |

R's `d[, "wage"]` with one name gives a vector and not a data frame; `select` always gives a
frame. Stata's `keep` changes the dataset in memory; `select` gives a new frame, and the frame
it was given is as it was.

## Related tasks
- To name the variables to remove instead, use `drop`.
- To keep some of the rows, use `where`.
- To give a variable another name, use `rename`.
- For the names of the variables, use `variables`.

## See also
drop, rename, where

## Examples
// Some variables, in the order named.
> let firms = dataset(firms)
> firms.select([size, firm]).head(3)
frame · 3 rows × 2 variables

      size   firm
  1      3      1
  2      5      2
  3      3      3

// One variable, by its bare name.
> firms.select(industry).head(2)
frame · 2 rows × 1 variable

      industry
  1   Agriculture
  2   Agriculture

// A range: every variable from one to another, by position.
> firms.select(industry..founded).variables()
industry
size
founded

// The variables of an analysis, taken from a wider frame.
> dataset(wages).select([worker, year, wage, educ]).head(3)
frame · 3 rows × 4 variables · panel worker × year

      worker   year    wage   educ
  1        1   2015   18.08     17
  2        1   2016   17.28     17
  3        1   2017   20.93     17
