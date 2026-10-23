# fitted

## Summary
The fitted values of a model

## Description
Gives the fitted values, the model's predictions for the rows it used, as a column of the fitted
frame's full length with `NA` in the rows the fit dropped, so that `wages.add(yhat = m.fitted())`
lines up, as R's `na.exclude` and Stata's `predict` do. `predict(newdata:)` gives the predictions
for other rows. A test's result has none and gives `NA`.

## Arguments
- fit: The result.

## Value
A `Column<Float>` of the fitted frame's length.

## Details
The fitted values are x′β computed row by row in a second pass over the data after the fit,
in the data's units under any weighting.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapter 3.

## See also
residuals, predict, coef

## Examples
> let wages = dataset(wages)
> let m = wages.reg(log(wage) ~ educ + exper + female)
> wages.add(yhat = m.fitted()).summary([yhat])
summary · frame

  variable        N   missing    mean      sd     min     max
  yhat       18,179       233   3.192   0.218   2.529   3.926
warning: 233 rows with NA in yhat were skipped (line 3)
