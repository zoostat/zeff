# rename

## Summary
Rename variables; the new name is on the left

## Description
A frame with the variables renamed as the definitions say, the new name on the left and the old
on the right: `rename(pay = wage)`. Nothing else changes: the columns, their order, their values
and their metadata are the same. The old name must be a variable of the frame (Z0101), and the new
name must not be another variable's (Z0401).

## Arguments
- data: The frame.
- ...: Definitions `new = old`, each naming the variable to rename without quotes.

## Value
A `Frame` with the same rows and the variables renamed.

## Missing values
Only the names change; the values, missing ones among them, are as they were.

## A common mistake
Writing the old name first, as Stata's `rename old new` does. The new name is on the left, as in every definition:

> dataset(firms).rename(size = employees)
error[Z0101]: no variable `employees` in `firms`
  = help: the variables of `firms` are: firm, industry, size, founded

Write `dataset(firms).rename(employees = size)`.

## Coming from R or Stata
| d.rename(pay = wage) | dplyr::rename(d, pay = wage), or names(d)[names(d) == "wage"] <- "pay" | rename wage pay |

Stata writes the old name first. Here, as in dplyr, the new name is on the left of `=`, the
side a name is given on in `let`, `add` and `frame`.

## Related tasks
- To give a variable a label that tables and plots print, use `label`.
- To keep some of the variables, use `select`.
- For the names of the variables, use `variables`.
- To change the values of a variable, use `update`.

## See also
select, drop, variables

## Examples
// The new name is on the left.
> let firms = dataset(firms)
> firms.rename(employees = size).head(3)
frame · 3 rows × 4 variables

      firm   industry      employees   founded
  1      1   Agriculture           3   1990-11-06
  2      2   Agriculture           5   1957-07-02
  3      3   Agriculture           3   1998-11-07

// Several at once.
> firms.rename(id = firm, sector = industry).variables()
id
sector
size
founded

// A name that is not a plain Zeff name, in backticks.
> firms.rename(`date founded` = founded).variables()
firm
industry
size
date founded
