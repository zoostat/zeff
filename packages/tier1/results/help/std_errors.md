# std_errors

## Summary
Standard errors of the estimates

## Description
Gives the standard errors of a fitted model, under the kind of standard error the fit was asked
for: every term with an estimate as a mapping from term name to value, in the model's order, or
one value when a term is named. A term omitted as collinear has none and gives `NA`; a test's
result has no standard errors and gives `NA`. Like `sqrt(diag(vcov(fit)))` in R.

## Arguments
- fit: The result.
- term: A term of the model, a bare name or text: `educ`, `industry=Retail`, `educ:female`.

## Value
A `Mapping` from term name to standard error, or a `Float` when a term is named.

## See also
p_values, ci, estimates, summary

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.std_errors()
(intercept) => 0.0220778
educ => 0.00136477
exper => 0.000451384
female => 0.00642367
> m.std_errors(educ)
0.00136477
