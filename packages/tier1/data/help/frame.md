# frame

## Summary
Create a frame from named lists of values

## Description
Builds a frame in the script itself, one column per definition: `frame(wage = [23.4, 19.0], female
= [true, false])`. Each list holds values of one type, Bool, Int, Float or Str, with `NA` for a
missing value; a list of Int and Float values is a Float column, as section 3.7 of the Language
Reference joins the two. Every list has the same number of values, so the frame is rectangular
(Z0217 otherwise), and a list with no value, or none but `NA`, has no type and is refused (Z0219).

Unlike R's `data.frame`, text never becomes a category, and a list of one value is a column of one
row, not a value recycled to the length of the others.

## Arguments
- ...: Each `name = [values]` defines a column called `name`, in the order written. A name that is
  not a plain Zeff name is written in backticks.

## Value
A `Frame` with the columns defined, in order.

## See also
import, describe, mean

## Examples
> let d = frame(wage = [23.4, 19.0, 31.2], female = [true, false, true])
> d
d · 3 rows × 2 variables

      wage   female
  1   23.4   true
  2     19   false
  3   31.2   true
> d.mean(wage)
24.5333
> frame(x = [1, 2.5, NA]).mean(x)
1.75
warning: 1 row with NA in x was skipped (line 4)
