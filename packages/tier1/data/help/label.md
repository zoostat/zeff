# label

## Summary
Set a variable's label, or the labels its codes stand for

## In brief
- `d.label(wage, "Hourly wage")`: the variable's label, the description it carries beside its name.
- `d.label(sex, [0 => "male", 1 => "female"])`: value labels, the text that the codes of an Int variable stand for.
- `d.label(wage, "")`: the label removed.

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

## Missing values
A label is said of a variable or of a code, never of `NA`: a missing value stays `NA` and prints as `NA`.

## A common mistake
Giving value labels to a category. A category's levels are its labels already; value labels are for the codes of an Int variable:

> dataset(firms).label(industry, [1 => "Farming"])
error[Z0401]: value labels name the codes of an Int variable, but `industry` is a Category variable
  = help: a category's levels are its labels; `recode(industry, ["old" => "new"])` renames them

Rename the level: `dataset(firms).recode(industry, ["Agriculture" => "Farming"])`.

## Coming from R or Stata
| d.label(wage, "Hourly wage") | attr(d$wage, "label") <- "Hourly wage" | label variable wage "Hourly wage" |
| d.label(sex, [0 => "male", 1 => "female"]) | d$sex <- factor(d$sex, levels = c(0, 1), labels = c("male", "female")) | label define sexl 0 "male" 1 "female", then label values sex sexl |

In R a labelled code is usually made a factor, which no longer holds the numbers; here, as in
Stata, the variable keeps its codes and shows the labels. Stata names a set of labels and
attaches it to the variable in a second step; here the mapping is given with the variable.

## Related tasks
- To make a category of a variable, use `category`.
- To rename the levels of a category, or to move codes, use `recode`.
- To change the name of a variable, use `rename`.
- To see which variables carry value labels, use `describe`.

## See also
category, recode, describe, rename

## Examples
// Value labels on a code read from a file: the variable keeps its numbers and shows the labels.
> let staff = import("zeff:staff.csv").select([employee, region, offer, training])
> let s = staff.label(offer, [0 => "no", 1 => "yes"])
> s.head(3)
note: zeff:staff.csv read as CSV: 2,400 rows × 9 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
frame · 3 rows × 4 variables

      employee   region   offer   training
  1          1   South    yes            1
  2          2   South    no             0
  3          3   North    no             1

// `describe` marks the variable `Int (lab)`.
> s.describe()
s · 2,400 rows × 4 variables

  variable   type        non-missing   missing   example
  employee   Int               2,400         0   1
  region     Str               2,400         0   South
  offer      Int (lab)         2,400         0   yes
  training   Int               2,400         0   1

// A category made of the labelled variable has the labels as its levels.
> levels(category(s["offer"]))
no
yes

// A variable label: the values print as before.
> staff.label(training, "Took the course").head(2)
frame · 2 rows × 4 variables

      employee   region   offer   training
  1          1   South        1          1
  2          2   South        0          0
