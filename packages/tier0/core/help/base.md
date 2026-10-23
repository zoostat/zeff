# base

## Summary
The reference level of a category

## Description
Moves one level of a category to the front and keeps the others in their order, so that it is the
base level: the level the indicators of a model are measured against, as R's `relevel` and Stata's
`ib#.` do. Inside a formula, `base(industry, "Retail")` names the base level of that term (section
7.1 of the Language Reference); on its own it returns the relevelled category, its order, contrast
and labels kept. A level the category does not have is an error that lists the levels and names the nearest one.

## Arguments
- var: A category column; inside a frame method or a formula, the variable's name.
- level: The level to put first, in quotes.

## Value
The category with `level` first.

## See also
category, levels

## Examples
> let staff = import("data/staff.csv")
note: data/staff.csv read as CSV: 4 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> let dept = category(staff["dept"])
> levels(base(dept, "Sales"))
Sales
HR
IT
> levels(dept)
HR
IT
Sales
