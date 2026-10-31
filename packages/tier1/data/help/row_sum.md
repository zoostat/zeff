# row_sum

## Summary
Sum across variables in each row

## Description
The sum of several variables within each row: for each row, the values present among the
variables named are added, and a row with no value present is `NA`, never 0, since a total of
nothing is not known (R's `rowSums(x, na.rm = TRUE)` and Stata's `egen rowtotal()` give 0 there,
which the page of each says to beware of). It stands inside `add` or `update`, where the frame's
variables are in scope, and takes them as a list, `[q1, q2, q3]`, or a range by position,
`[eng1..eng6]`. The variables must be Ints or Floats; a Bool is refused with the conversion that
turns it into 0 and 1. The result is a column of Floats, computed with the compensated summation
every sum here uses, so `row_sum([x, x, x])` is `3 * x` exactly.

## Arguments
- vars: The variables to add, a list of names or a range.

## Value
A `Column<Float>`, `NA` where no value is present.

## Details
For each row, the compensated sum of the values present; Int variables are read exactly as Floats below 2^53.

## References
R Core Team, the `rowSums` function of the base package, with `na.rm = TRUE`.
Stata, `egen` with the `rowtotal()` function.

## See also
row_mean, sum, n_missing, fill_missing

## Examples
> let d = frame(q1 = [5, 4, NA], q2 = [3, NA, NA], q3 = [4, 4, NA])
> d.add(total = row_sum([q1, q2, q3]))
frame · 3 rows × 4 variables

      q1   q2   q3   total
  1    5    3    4      12
  2    4   NA    4       8
  3   NA   NA   NA      NA
