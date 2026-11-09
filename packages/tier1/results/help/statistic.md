# statistic

## Summary
The statistic of a test

## Description
Gives the test statistic of a test's result as a number: the t, z, chi-squared or H the
printout shows before its p-value. A test without a statistic of its own, such as the exact
`fisher_test`, gives `NA`. Like `t$statistic` in R and `r(t)` or `r(chi2)` in Stata.

## Arguments
- t: A test's result.

## Value
A `Float`, or `NA` for a test without a statistic.

## See also
df, p_values, effect_size, estimates

## Examples
> let t = dataset(wages).t_test(educ, group: female)
> t.statistic()
-1.1778
> dataset(wages).fisher_test(union, female).statistic()
NA
