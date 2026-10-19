# rename

## Summary
Rename variables; the new name is on the left

## Description
A frame with the variables renamed as the definitions say, the new name on the left and the old
on the right: `rename(pay = wage)`. Nothing else changes: the columns, their order, their values
and their metadata are the same. The old name must be a variable of the frame (Z0101), and the new
name must not be another variable's (Z0401). Like dplyr's `rename` and Stata's `rename old new`
with the sides swapped.

## Arguments
- data: The frame.
- ...: Definitions `new = old`, each naming the variable to rename without quotes.

## Value
A `Frame` with the same rows and the variables renamed.

## See also
select, drop, variables

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.rename(pay = wage, worker = id).variables()
worker
pay
hours
