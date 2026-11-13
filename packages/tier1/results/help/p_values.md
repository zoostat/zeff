# p_values

## Summary
p-values of the estimates

## Description
Gives the two-sided p-values of a fitted model's terms, from the t table of the fit: every term
with an estimate as a mapping from term name to value, in the model's order, or one value when a
term is named; a term omitted as collinear gives `NA`. A test's result gives its single p-value. Where a
test's result prints a table of several rows, the p-value of one row is read by naming it: a
term of a factorial `anova`, `p_values("industry")`, or a comparison of `posthoc`, named as
its table prints it, `p_values("a − b")`, where the minus sign may be typed as a hyphen;
without a name it is the p-value of the whole test. Like `coef(summary(fit))[, 4]` in R; one
row of a table is the `Pr(>F)` column of `car::Anova()` or the `p adj` column of `TukeyHSD()`.

## Arguments
- fit: The result.
- term: A term of the model, a bare name or text: `educ`, `industry=Retail`, `educ:female`. On a test's result, a row of its table: a term of a factorial `anova` or a comparison of `posthoc`.

## Value
A `Mapping` from term name to p-value, or a `Float` when a term is named or the result is a
test's. A name that is no term of the fit, or no row of the test's table, is an error (Z0414)
that lists them.

## See also
std_errors, ci, estimates, summary, statistic, df, anova, posthoc

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.p_values(educ)
0
> let a = dataset(wages).add(lwage = log(wage)).anova(lwage ~ industry * category(female) + educ)
note: 211 rows with NA in a model variable dropped (line 3)
> a.p_values("industry:female")
0.00031527
> a.p_values(educ)
0
