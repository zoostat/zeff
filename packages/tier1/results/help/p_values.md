# p_values

## Summary
p-values of the estimates

## Description
Gives the two-sided p-values of a fitted model's terms, from the t table of the fit: every term
with an estimate as a mapping from term name to value, in the model's order, or one value when a
term is named; a term omitted as collinear gives `NA`. A test's result gives its single p-value.
Like `coef(summary(fit))[, 4]` in R.

## Arguments
- fit: The result.
- term: A term of the model, a bare name or text: `educ`, `industry=Retail`, `educ:female`.

## Value
A `Mapping` from term name to p-value, or a `Float` when a term is named or the result is a
test's.

## See also
std_errors, ci, estimates, summary

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.p_values(educ)
0
