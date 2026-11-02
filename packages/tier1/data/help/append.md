# append

## Summary
Stack another frame's rows below

## Description
The rows of `frame` placed below the rows of `data`, the columns matched by name: a column that
one frame lacks is filled with `NA` on the other's rows, and the note names it. A column of Int
in one frame and Float in the other becomes Float; a category's levels are the union of the two,
in the order first seen; any other difference of type is an error (Z0609), since the rows of one
column have one type. The left frame's rows come first, in their order, then the right's. Labels
and value labels are kept where the left frame has the column, and taken from the right for a
column only it has. A panel declaration survives when both frames carry the same one. Like R's
`rbind` on matching names and Stata's `append`.

## Arguments
- data: The frame whose rows come first.
- frame: The frame whose rows are added below.

## Value
A frame with the columns of both, in the left frame's order then the right's new ones, and
every row of both.

## Details
Exact: values are moved, not computed; an Int beside a Float is the same number as a Float.

## See also
merge, to_long, to_wide, frame

## Examples
> let this_year = frame(firm = [1, 2], revenue = [12.5, 8], region = ["North", "South"])
> let last_year = frame(firm = [1, 3], revenue = [11, 7])
> this_year.append(last_year)
frame · 4 rows × 3 variables

      firm   revenue   region
  1      1      12.5   North
  2      2         8   South
  3      1        11   NA
  4      3         7   NA
note: append: 2 rows added below 2 rows; `region` only in this_year, filled with NA below (line 3)
