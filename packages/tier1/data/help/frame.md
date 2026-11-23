# frame

## Summary
Create a frame from named lists of values

## Description
Builds a frame in the script itself, one column per definition: `frame(wage = [23.4, 19.0], female
= [true, false])`. Each list holds values of one type, Bool, Int, Float or Str, with `NA` for a
missing value; a list of Int and Float values is a Float column, as section 3.7 of the Language
Reference joins the two. Every list has the same number of values, so the frame is rectangular
(Z0217 otherwise), and a list with no value, or none but `NA`, has no type and is refused (Z0219).

Text never becomes a category, and a list of one value is a column of one row.

## Arguments
- ...: Each `name = [values]` defines a column called `name`, in the order written. A name that is
  not a plain Zeff name is written in backticks.

## Value
A `Frame` with the columns defined, in order.

## Missing values
`NA` in a list is a missing value of the column's type, which the other values of the list give. A list with no value, or with nothing but `NA`, has no type to give its column and is refused (Z0219).

## A common mistake
Giving lists of different lengths, as when a value that is not known is left out. Nothing is repeated to fill a column:

> frame(id = [1, 2, 3], wage = [23.4, 19.0])
error[Z0217]: `frame` needs lists of one length: `id` has 3 values and `wage` has 2
  = help: give every column the same number of values; write `NA` for a value that is missing

Write `NA` where a value is missing: `frame(id = [1, 2, 3], wage = [23.4, 19.0, NA])` has three rows.

## Coming from R or Stata
| frame(id = [1, 2], wage = [23.4, 19.0]) | data.frame(id = c(1, 2), wage = c(23.4, 19.0)) | input id wage, a line for each row, end |

R's `data.frame` repeats a value given once down the column, so `data.frame(id = 1:3, year = 2026)`
has three rows; here every list is written out in full. Before R 4.0 `data.frame` turned text into
a factor unless told not to; here text is text, and `category` makes a category when one is
wanted. Stata holds one dataset in memory and `input` types rows into it; here a frame is a
value with a name, and a script may hold as many as it needs.

## Related tasks
- To read data from a file, use `import`.
- For a dataset that ships with Zeff, use `dataset`.
- To add a variable to a frame, use `add`.
- To put the rows of one frame under those of another, use `append`.

## See also
import, describe, mean

## Examples
// One column for each definition, in the order written.
> frame(id = [1, 2, 3], wage = [23.4, 19.0, 31.2], female = [true, false, true])
frame · 3 rows × 3 variables

      id   wage   female
  1    1   23.4   true
  2    2     19   false
  3    3   31.2   true

// `NA` is a missing value of the column's type; whole numbers beside decimals make a Float column.
> frame(hours = [40, NA, 35.5], union = ["yes", NA, "no"])
frame · 3 rows × 2 variables

      hours   union
  1      40   yes
  2      NA   NA
  3    35.5   no

// A name that is not a plain Zeff name goes in backticks.
> frame(`hourly wage` = [23.4, 19.0])
frame · 2 rows × 1 variable

      hourly wage
  1          23.4
  2            19

// Rows typed in for a model fitted on the shipped wages: the predicted log wage at 12 and at 16 years of schooling.
> let m = dataset(wages).reg(log(wage) ~ educ + exper)
> m.predict(newdata: frame(educ = [12, 16], exper = [10, 10]))
Column<Float> · 2 values
2.9664
3.27044
