# levels

## Summary
A category's levels, in order

## Description
The levels of a category column as a list of text, in the order the category has them: the order
`category` gave or `levels:` named, which is the order of the indicators in a model, with the
first level the base unless `base` chooses another. R has `levels`.

## Arguments
- var: A category column; inside a frame method, the variable's name.

## Value
A `List<Str>`.

## See also
category, base, n_distinct

## Examples
> let staff = import("data/staff.csv")
note: data/staff.csv read as CSV: 4 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> levels(category(staff["dept"]))
HR
IT
Sales
> levels(category(staff["grade"]))
1
2
3
