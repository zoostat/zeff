# row_mean

## Summary
Mean across variables in each row

## Description
The mean of several variables within each row, such as a scale score from its items: for each
row, the values present among the variables named are averaged, and a row with fewer than
`min_valid:` values present is `NA`. It stands inside `add` or `update`, where the frame's
variables are in scope, and takes them as a list, `[q1, q2, q3]`, or a range by position,
`[eng1..eng6]` (section 3.7 of the Language Reference, "Variable ranges"). The variables must be
Ints or Floats; a Bool is refused with the conversion that turns it into 0 and 1.

The mean is over the values present, as R's `rowMeans(x, na.rm = TRUE)` and Stata's
`egen rowmean()` compute it, with the compensated summation every mean here uses; `min_valid:`
is the rule psychometrics applies to a scale with missing items (so many of the items answered,
or no score), which the R and Stata functions leave to the user.

## Arguments
- vars: The variables to average, a list of names or a range.
- min_valid: How many values a row needs for a mean; 1 unless given.

## Value
A `Column<Float>`, `NA` where too few values are present.

## Details
For each row, the compensated sum of the values present divided by their count; a row's count below `min_valid:` gives `NA`. Int variables are read exactly as Floats below 2^53.

## References
R Core Team, the `rowMeans` function of the base package, with `na.rm = TRUE`.
Stata, `egen` with the `rowmean()` function.

## See also
row_sum, mean, n_missing, fill_missing

## Examples
> let d = frame(q1 = [5, 4, NA], q2 = [3, NA, NA], q3 = [4, 4, 2])
> d.add(score = row_mean([q1, q2, q3]))
frame · 3 rows × 4 variables

      q1   q2   q3   score
  1    5    3    4       4
  2    4   NA    4       4
  3   NA   NA    2       2
> d.add(score = row_mean([q1..q3], min_valid: 2))
frame · 3 rows × 4 variables

      q1   q2   q3   score
  1    5    3    4       4
  2    4   NA    4       4
  3   NA   NA    2      NA
