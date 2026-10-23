# category

## Summary
Make a categorical variable, with its levels in order

## Description
Turns a column into a category: text gets its distinct values as levels in code-point order; a
numeric variable its distinct values, written as they print, in numeric order; an Int with value
labels its labels, in the order of the codes; and a category keeps its levels. `levels:` names the
levels and their order instead, as text or as the numbers a numeric variable holds, and a value
that is none of them becomes `NA` with a warning giving the count and up to three of the distinct
values (W0203). `ordered: true` gives the levels an order, so that `<` compares them by position and the
variable's `measure` is `ordinal`; `contrast:` sets how a model codes the indicators (section 4.2
of the Language Reference): `treatment` against the first level, `sum` as deviations from the
mean, `helmert` as each level against the mean of those before it, `poly` as polynomial trends over
ordered levels. In a model the indicator columns are named by the variable and the level each
codes, `industry=Retail`: under `treatment` and `sum` every level but the base (R names them
`industryRetail` and `industry1`), under `helmert` the level compared with the mean of those before
it (R: `industry1`), and under `poly` the trend, `industry=linear`, `industry=quadratic`,
`industry=cubic`, `industry=degree4` and on (R: `.L`, `.Q`, `.C`, `^4`). A category of more than
10,000 levels is warned about (W0602), since an identifier was probably categorised. R's `factor`
sorts levels by the locale's collation; Zeff by code point, the same on every machine. Stata's
`encode` orders labels alphabetically as well.

## Arguments
- var: A text, Int, Float or category column; inside `add`, the variable's name.
- levels: The levels in the order they should have; omitted, the distinct values in their sorted order.
- ordered: Whether the levels are ordered.
- contrast: How a model codes the indicators.

## Value
A category column, with no metadata but the order and the contrast.

## Details
Exact: the levels and codes follow the rules above, with no guess. A level with no rows is still a
level, and a value outside `levels:` is never added silently.

## See also
levels, base, to, type_of

## Examples
> let staff = import("data/staff.csv")
note: data/staff.csv read as CSV: 4 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> let s = staff.add(dept = category(dept), grade = category(grade, levels: [1, 2, 3], ordered: true))
note: `dept` replaced the variable of that name (line 2)
note: `grade` replaced the variable of that name (line 2)
> s.describe()
s · 4 rows × 3 variables

  variable   type       non-missing   missing   example
  name       Str                  4         0   Ann
  dept       Category             4         0   Sales (3 levels)
  grade      Category             4         0   2 (3 levels)
> levels(s["dept"])
HR
IT
Sales
> s.where(grade >= "2").n_rows()
3
> staff.add(dept = category(dept, levels: ["Sales", "IT"])).describe()
frame · 4 rows × 3 variables

  variable   type       non-missing   missing   example
  name       Str                  4         0   Ann
  dept       Category             3         1   Sales (2 levels)
  grade      Int                  4         0   2
warning: 1 value is none of the levels given and became NA: "HR" (line 6)
  = help: add the value to `levels:`, or recode it first; `is_missing` finds the NA afterwards
note: `dept` replaced the variable of that name (line 6)
