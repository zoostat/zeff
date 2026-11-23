# recode

## Summary
Map a variable's values, merge a category's levels, or move labelled codes

## In brief
- `d.recode(industry, ["Mining" => "Other", "Utilities" => "Other"])`: levels of a category merged or renamed.
- `d.recode(grade, [1 => 2, 3 => 4])`: values of a number, a text or a Bool variable given other values.
- `d.recode(sex, [0 => 2])`: on an Int with value labels, the codes moved and their labels with them.
- `d.recode(hours, [NA => 0])`: `NA` as a key fills the gaps; as a value it makes rows missing.

## Description
A frame with one variable recoded by a mapping of old to new in every row that holds an old
value; the other rows, the variable's type and its metadata are kept. On a category the keys are
levels: a merged level stands where the first of its old levels stood and the other old levels
vanish, a new name is a new level in that place, and `NA` as a value removes a level and makes its
rows `NA`, as forcats' `fct_collapse` and `fct_recode` do. On an Int, Float, text or Bool variable
the values are mapped by identity (section 3.7 of the Language Reference), so `NA` may be a key,
`[NA => 0]` fills the gaps, and `-0.0` and `0.0` are one value; the new values must be of the
variable's type, or `NA`. On an Int with value labels the codes are mapped and their labels follow
them: `[0 => 2]` moves the label of 0 to 2, and a code merged into another takes the label of the
first of its old codes, as Stata's `recode` keeps the labels. A key that is no level or value of
the variable is a warning naming it (W0206), as forcats warns, and the other keys are applied. A
note says how many rows changed and, for a category, how many levels remain.

## Arguments
- data: The frame.
- var: The variable, by name.
- mapping: `[old => new, ...]`, each old value or level once; `NA` as a key matches the missing
  rows, and as a value makes the rows missing.

## Value
A `Frame` with the same rows and variables, the variable recoded.

## Missing values
`NA` may be a key, `[NA => 0]`, which gives the missing rows a value, and a value, `[99 => NA]`, which makes rows missing; a row that is `NA` and is not named as a key stays `NA`. On a category, `NA` as a value removes the level.

## A common mistake
Giving a new value of another type. `recode` keeps the type of the variable:

> dataset(firms).recode(size, [0 => "none"])
error[Z0401]: `size` is an Int variable, but a value of the mapping is a Str
  = help: `recode` keeps a variable's type, so the keys and values are whole numbers or `NA`; `add(size = ...)` makes a variable of another type

Keep to the type, `recode(size, [0 => NA])`, or make a text variable with `add`.

## Coming from R or Stata
| d.recode(grade, [1 => 2, 3 => 4]) | dplyr::recode(d$grade, `1` = 2, `3` = 4) | recode grade (1 = 2) (3 = 4) |
| d.recode(industry, ["Mining" => "Other", "Utilities" => "Other"]) | forcats::fct_collapse(d$industry, Other = c("Mining", "Utilities")) | recode industry (2 3 = 13), on its codes |
| d.recode(hours, [NA => 0]) | d$hours[is.na(d$hours)] <- 0 | replace hours = 0 if missing(hours) |

forcats writes the new level first, `Other = c(...)`; here, as in Stata, the old value is on the
left of the arrow. Stata's `recode` works on numbers alone, so a labelled variable is recoded by
its codes; here a category is recoded by its levels.

## Related tasks
- To change values by a condition and not by a list of values, use `update`.
- To give the codes of an Int variable their labels, use `label`.
- To make a category, or to set the order of its levels, use `category`.
- To fill missing values by a rule, use `fill_missing`.

## See also
category, levels, base, label, update

## Examples
// Two levels of a category merged into one, which stands where the first of them stood.
> let wages = dataset(wages)
> let w = wages.recode(industry, ["Mining" => "Other", "Utilities" => "Other"])
> levels(w["industry"])
note: `industry` recoded: 570 rows changed; 11 levels, from 12 (line 2)
Agriculture
Other
Construction
Manufacturing
Retail
Transport
Finance
Services
Education
Health
Public administration

// On an Int with value labels the codes move and their labels follow.
> levels(category(wages.recode(female, [0 => 2])["female"]))
female
male
note: `female` recoded: 9,491 rows changed; the value labels follow their codes (line 4)

// Values of a number.
> let staff = import("zeff:staff.csv").select([employee, site, region])
> staff.recode(site, [1 => 10, 2 => 10]).head(3)
note: zeff:staff.csv read as CSV: 2,400 rows × 9 variables, types guessed; CSV carries no labels, value labels or categories (line 5)
frame · 3 rows × 3 variables

      employee   site   region
  1          1      9   South
  2          2      7   South
  3          3     10   North
note: `site` recoded: 405 rows changed (line 6)

// A key that is no value of the variable is a warning, and the other keys are applied.
> staff.recode(region, ["South" => "S", "Sout" => "S"]).head(2)
frame · 2 rows × 3 variables

      employee   site   region
  1          1      9   S
  2          2      7   S
warning[W0206]: `recode`: "Sout" is not a value of `region` (line 7)
  = help: check the spelling or the type of the key; the other keys were applied
note: `region` recoded: 599 rows changed (line 7)

// `NA` as a value makes rows missing.
> dataset(firms).recode(size, [0 => NA]).where(is_missing(size)).n_rows()
3
note: `size` recoded: 3 rows changed (line 8)
