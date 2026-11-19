# category

## Summary
Make a categorical variable, with its levels in order

## In brief
- `category(x)`: a category whose levels are the distinct values, in sorted order.
- `category(x, levels: ["low", "mid", "high"], ordered: true)`: the levels named, in that order,
  and ordered, so that `<` compares them.
- `category(x, contrast: sum)`: the same category, coded in a model as deviations from the mean.

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

## Missing values
`NA` stays `NA` and is not a level. A value that is none of the `levels:` given becomes `NA`, and warning W0203 gives the count and up to three of the values; no row is dropped.

## Details
Exact: the levels and codes follow the rules above, with no guess. A level with no rows is still a
level, and a value outside `levels:` is never added silently.

## A common mistake
Naming some of the levels and forgetting one. A value that is none of them is not added silently: it becomes `NA`, and the warning names it:

> category(staff["dept"], levels: ["Sales", "IT"])
Column<Category> · 4 values
Sales
IT
Sales
NA
warning[W0203]: 1 value is none of the levels given and became NA: "HR" (line 6)
  = help: add the value to `levels:`, or recode it first; `is_missing` finds the NA afterwards

Name every level, `levels: ["Sales", "IT", "HR"]`, or leave `levels:` out to take them all.

## Coming from R or Stata
| category(x) | factor(x) | encode x, generate(x2) |
| category(x, levels: ["a", "b"]) | factor(x, levels = c("a", "b")) | label define, then encode x, label() |
| category(x, ordered: true) | factor(x, ordered = TRUE) | no ordered type |
| category(x, contrast: sum) | contrasts(x) <- contr.sum(k) | no single function |

R's `factor` turns a value outside `levels` into `NA` without a word; here that is counted and
shown. R sorts the levels by the computer's language setting; here by code point, the same on
every machine. In a model the indicator of a level is named `industry=Retail`, where R writes
`industryRetail`.

## Related tasks
- To see a category's levels, use `levels`.
- To choose the level a model measures against, use `base`.
- For labels on whole-number codes, use `label`.

## See also
levels, base, to, type_of

## Examples
// Text becomes a category: its levels are the distinct values, sorted.
> let staff = frame(name = ["Ann", "Bob", "Cy", "Dee"], dept = ["Sales", "IT", "Sales", "HR"], grade = [2, 1, 3, 2])
> levels(category(staff["dept"]))
HR
IT
Sales

// Levels named in their order, and ordered, on a variable inside `add`.
> let s = staff.add(grade = category(grade, levels: [1, 2, 3], ordered: true))
note: `grade` replaced the variable of that name (line 3)

// An ordered category compares by position.
> s.where(grade >= "2").n_rows()
3

// On the shipped wages, coded as deviations from the mean in a model: the indicator is named by the variable and its level.
> dataset(wages).add(female = category(female, contrast: sum)).reg(log(wage) ~ female)
reg · log(wage) ~ female
N = 18,201 (211 rows dropped for NA in model variables)
standard errors: classical

  term            estimate   std.error   t value   p value   95% CI
  (intercept)       3.1899      0.0036    896.31    <0.001   [3.1829, 3.1969]
  female=female    -0.0689      0.0036    -19.35    <0.001   [-0.0758, -0.0619]

R² 0.020 · adjusted R² 0.020
note: `female` replaced the variable of that name (line 5)
