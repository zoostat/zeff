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
column only it has. A panel declaration survives when both frames carry the same one.
A chain of appends in one statement, `a.append(b).append(c)`, or a loop that stacks a frame in
each pass, gives one note for the chain: how many times, the rows added in all, and what was
filled.

## Arguments
- data: The frame whose rows come first.
- frame: The frame whose rows are added below.

## Value
A frame with the columns of both, in the left frame's order then the right's new ones, and
every row of both.

## Missing values
A column that one frame lacks is `NA` on the rows of that frame, and the note names it. Values that are missing stay missing.

## Details
Exact: values are moved, not computed; an Int beside a Float is the same number as a Float.

## A common mistake
Stacking frames in which one column has two types, as when an identifier was read as a number in one file and as text in the other:

> this_year.append(frame(firm = ["3"], revenue = [5.5]))
error[Z0609]: `firm` is a column of Int in this_year and of Str in the frame
  = help: the rows of one column have one type; convert one side first with `to`, or `drop` the column from one frame

Convert one side first with `to`, so that the column has one type in both.

## Coming from R or Stata
| a.append(b) | rbind(a, b), or dplyr::bind_rows(a, b) | append using b |

R's `rbind` refuses two data frames whose columns differ; `bind_rows` fills the missing column
with `NA`, as here. Stata's `append using` reads the second dataset from a file and fills in
the same way.

## Related tasks
- To put the columns of two frames side by side by a key, use `merge`.
- To turn repeated columns into rows of one frame, use `to_long`.
- To type a small frame of rows to add, use `frame`.

## See also
merge, to_long, to_wide, frame

## Examples
// Two years of the shipped wages, one under the other.
> let wages = dataset(wages)
> let y2019 = wages.where(year == 2019)
> let y2020 = wages.where(year == 2020)
> y2019.append(y2020).n_rows()
2620
note: append: 1,102 rows added below 1,518 rows (line 4)

// A column that one frame lacks is filled with `NA`, and the note names it.
> let this_year = frame(firm = [1, 2], revenue = [12.5, 8], region = ["North", "South"])
> let last_year = frame(firm = [1, 3], revenue = [11, 7])
> this_year.append(last_year)
frame · 4 rows × 3 variables

      firm   revenue   region
  1      1      12.5   North
  2      2         8   South
  3      1        11   NA
  4      3         7   NA
note: append: 2 rows added below 2 rows; `region` only in this_year, filled with NA below (line 7)

// A chain of appends has one note.
> y2019.append(y2020).append(wages.where(year == 2021)).n_rows()
3183
note: append, 2 times: 1,665 rows added below 1,518 rows (line 8)
