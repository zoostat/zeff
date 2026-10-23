# vcov

## Summary
The variance matrix of the estimates

## Description
Gives the variance–covariance matrix of a fitted model's coefficients, under the kind of standard
error the fit was asked for (classical, robust, HC2, HC3 or clustered), as a table over the terms
with an estimate; its diagonal is the squared standard errors. A test's result has none and gives
`NA`. Like `vcov` in R and `e(V)` in Stata.

## Arguments
- fit: The result.

## Value
A `Table` with one row and one column per term, titled with the kind of standard error.

## Details
The matrix is the one the fit's standard errors are the square roots of: σ²(XᵀX)⁻¹ under
classical errors, the sandwich of the scores and the bread under robust, HC2, HC3 and clustered
errors, as `reg` describes; it is symmetric by construction and printed with six significant
digits.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapters 3 and 5.

## See also
std_errors, coef, export

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.vcov()
vcov · classical

  term           (intercept)           educ          exper         female
  (intercept)    0.000487427   -2.81326e-05   -5.80141e-06   -2.11689e-05
  educ          -2.81326e-05    1.86261e-06    1.99764e-07    6.01044e-10
  exper         -5.80141e-06    1.99764e-07    2.03748e-07    7.60086e-08
  female        -2.11689e-05    6.01044e-10    7.60086e-08    4.12635e-05
