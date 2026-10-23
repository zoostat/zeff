# label

## Summary
Set a variable's label, or the labels its codes stand for

## Description
With text, sets the variable label: the description a variable carries beside its name, which
readers of Stata, SPSS and SAS files fill, Arrow files keep and Zemp's variable view shows; an
empty text removes it. With a mapping of whole numbers to text, sets the value labels of an Int
variable, as Stata's `label define` and `label values` do: the variable keeps its codes, and
frames, tables, `describe` and `category` show the labels instead, `describe` marking the variable
`Int (lab)`, as the shipped `wages` shows `female` as `male` and `female`. The mapping replaces any
value labels the variable had and is stored in code order; a code without a label prints as its
number, and a label may name a code no row holds. Value labels on a text, Float, Bool or category
variable are an error whose help names the way round: `category` for text, `recode` for a
category's levels.

## Arguments
- data: The frame.
- var: The variable, by name.
- labels: Text for the variable label, or `[code => "text", ...]` for value labels.

## Value
A `Frame` with the same rows and variables, the variable's metadata changed.

## See also
category, recode, describe, rename

## Examples
> let staff = import("data/staff.csv")
note: data/staff.csv read as CSV: 4 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> let s = staff.label(grade, [1 => "junior", 2 => "mid", 3 => "senior"])
> s
s · 4 rows × 3 variables

      name   dept    grade
  1   Ann    Sales   mid
  2   Bob    IT      junior
  3   Cy     Sales   senior
  4   Dee    HR      mid
> s.describe()
s · 4 rows × 3 variables

  variable   type        non-missing   missing   example
  name       Str                   4         0   Ann
  dept       Str                   4         0   Sales
  grade      Int (lab)             4         0   mid
> levels(category(s["grade"]))
junior
mid
senior
> staff.label(dept, "Department").head(2)
frame · 2 rows × 3 variables

      name   dept    grade
  1   Ann    Sales       2
  2   Bob    IT          1
