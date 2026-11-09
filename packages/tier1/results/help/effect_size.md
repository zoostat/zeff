# effect_size

## Summary
The effect size of a test

## Description
Gives the effect size a test's result printed, as a number: Hedges' g of a two-group
`t_test`. A test without an effect size gives `NA`; its interval, where it has one, is a row of
`ci()`. Like `effectsize::hedges_g()$Hedges_g` in R.

## Arguments
- t: A test's result.

## Value
A `Float`, or `NA` for a test without an effect size.

## See also
statistic, estimates, ci

## Examples
> let t = dataset(wages).t_test(educ, group: female)
> t.effect_size()
-0.0173753
> dataset(wages).chi2_test(union, female).effect_size()
NA
