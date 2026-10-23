# recode

## Summary
Map a variable's values, merge a category's levels, or move labelled codes

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

## See also
category, levels, base, label, update

## Examples
> let wages = dataset(wages)
> let w = wages.recode(industry, ["Mining" => "Other", "Utilities" => "Other"])
note: `industry` recoded: 570 rows changed; 11 levels, from 12 (line 2)
> levels(w["industry"])
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
> levels(category(wages.recode(female, [0 => 2])["female"]))
female
male
note: `female` recoded: 9,491 rows changed; the value labels follow their codes (line 4)
> let staff = import("data/staff.csv")
note: data/staff.csv read as CSV: 4 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 5)
> staff.recode(grade, [1 => 2, 3 => 4])
frame · 4 rows × 3 variables

      name   dept    grade
  1   Ann    Sales       2
  2   Bob    IT          2
  3   Cy     Sales       4
  4   Dee    HR          2
note: `grade` recoded: 2 rows changed (line 6)
> staff.recode(dept, ["Sales" => "Shop", "Marketing" => "Shop"]).head(1)
frame · 1 row × 3 variables

      name   dept   grade
  1   Ann    Shop       2
warning: `recode`: "Marketing" is not a value of `dept` (line 7)
  = help: check the spelling or the type of the key; the other keys were applied
note: `dept` recoded: 2 rows changed (line 7)
