# drop_missing

## Summary
Remove rows with NA in the variables and report how many

## Description
The rows with a value in every one of the variables named, in their order; a row with `NA` in
any of them is dropped. One note says how many rows went and which variables were looked at,
`228 rows with NA in educ or exper dropped`, or that no row had `NA` in them, so that a cleaning
step is never silent (section 4.1 of the Language Reference): `where(cond)` drops the rows where
its condition is `NA` with the same kind of note, and `drop_missing` is that filter written for
missingness itself. The variables are a list or a range by position; the whole frame is
`drop_missing([first..last])` with its first and last variable. Metadata and the frame's
declaration travel with the rows. Like R's `na.omit` on the subset of variables, and
`complete.cases`; Stata has no single command.

Dropping rows is a modelling decision as much as filling them (see `fill_missing`): the
reducers and the regressions skip `NA` on their own and say how many rows they skipped, so
`drop_missing` is for the cases where one sample should serve several analyses.

## Arguments
- data: The frame.
- vars: The variables that must be present, a list of names or a range.

## Value
A frame with the rows kept, in their order.

## See also
where, n_missing, fill_missing, is_missing, select

## Examples
> let d = frame(id = [1, 2, 3], x = [1, NA, 3], y = [NA, 2, 3])
> d.drop_missing([x, y])
frame · 1 row × 3 variables

      id   x   y
  1    3   3   3
note: 2 rows with NA in x or y dropped (line 2)
> d.drop_missing([id])
frame · 3 rows × 3 variables

      id    x    y
  1    1    1   NA
  2    2   NA    2
  3    3    3    3
note: no row had NA in id (line 3)
