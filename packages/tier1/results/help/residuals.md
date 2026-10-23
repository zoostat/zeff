# residuals

## Summary
The residuals of a fitted model

## Description
Gives the residuals, outcome less fitted value, as a column of the fitted frame's full length
with `NA` in the rows the fit dropped, so that `wages.add(res = m.residuals())` lines up, as R's
`na.exclude` and Stata's `predict, residuals` do. Under weights the residuals are those of the
data, not of the weighted rows. A test's result has none and gives `NA`.

## Arguments
- fit: The result.

## Value
A `Column<Float>` of the fitted frame's length.

## Details
The residuals are y − x′β computed row by row in a second pass over the data after the fit,
in the data's units under any weighting; the fit checks them orthogonal to the regressors
before it returns.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapter 3.

## See also
fitted, predict, influence

## Examples
> let wages = dataset(wages)
> let m = wages.reg(log(wage) ~ educ + exper + female)
> wages.add(res = m.residuals()).summary([res])
summary · frame

  variable        N   missing    mean      sd      min     max
  res        18,179       233   0.000   0.433   -2.006   1.530
warning: 233 rows with NA in res were skipped (line 3)
