# statistic

## Summary
The statistic of a test

## Description
Gives the test statistic of a test's result as a number: the t, z, chi-squared or H the
printout shows before its p-value. A test without a statistic of its own, such as the exact
`fisher_test`, gives `NA`. Where the result prints a table of several rows, the statistic of one
row is read by naming it: the F of a term of a factorial `anova`, `statistic("industry")`, or
the t of a comparison of `posthoc`, named as its table prints it, `statistic("a − b")`, where
the minus sign may be typed as a hyphen; without a name the statistic is that of the whole
test, the F of the whole model for a factorial `anova`. Like `t$statistic` in R and `r(t)` or
`r(chi2)` in Stata; one row is the `F value` column of `car::Anova()` in R.

## Arguments
- t: A test's result.
- term: A row of the result's table, a bare name or text: a term of a factorial `anova`, `dose` or `"dose:diet"`, or a comparison of `posthoc` as its table prints it, `"low − mid"`. Without it, the figure of the whole test.

## Value
A `Float`, or `NA` for a test without a statistic. A name that is no row of the table is an
error (Z0414) that lists the rows.

## See also
df, p_values, effect_size, estimates, anova, posthoc

## Examples
> let t = dataset(wages).t_test(educ, group: female)
> t.statistic()
-1.1778
> dataset(wages).fisher_test(union, female).statistic()
NA
> let a = dataset(wages).add(lwage = log(wage)).anova(lwage ~ industry * category(female) + educ)
note: 211 rows with NA in a model variable dropped (line 4)
> a.statistic()
101.468
> a.statistic("industry")
34.1015
> a.statistic(educ)
1587.37
