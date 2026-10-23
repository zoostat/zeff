# coef

## Summary
The estimated coefficients

## Description
Gives the coefficients of a fitted model: every term with an estimate as a mapping from term
name to value, in the model's order, or one value when a term is named. A term omitted as
collinear has no coefficient and gives `NA`; a test's result has none and gives `NA`. Like
`coef` in R and `e(b)` in Stata.

## Arguments
- fit: The result.
- term: A term of the model, a bare name or text: `educ`, `industry=Retail`, `educ:female`.

## Value
A `Mapping` from term name to coefficient, or a `Float` when a term is named.

## Details
The coefficients are those the fit printed, in full precision: the least-squares solution of
the scaled design by back substitution from its R factor, unscaled (see `reg`). A term omitted
as collinear has none.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapter 3.

## See also
std_errors, ci, estimates, vcov

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.coef()
(intercept) => 1.89982
educ => 0.0760082
exper => 0.02202
female => -0.132924
> m.coef(educ)
0.0760082
