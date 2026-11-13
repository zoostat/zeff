# effect_size

## Summary
The effect size of a test

## Description
Gives the effect size a test's result printed, as a number: Hedges' g of a two-group
`t_test`. A test without an effect size gives `NA`; its interval, where it has one, is a row of
`ci()`. Where the result prints a table of several rows, the effect size of one row is read by
naming it: the partial eta squared of a term of a factorial `anova`, `effect_size("educ")`, or
the measure its `effect:` asked for; without a name such a table has no single effect size and
the value is `NA`, as it is for a comparison of `posthoc`. Like
`effectsize::hedges_g()$Hedges_g` in R; one row is `effectsize::eta_squared(partial = TRUE)`.

## Arguments
- t: A test's result.
- term: A row of the result's table, a bare name or text: a term of a factorial `anova`, `dose` or `"dose:diet"`, or a comparison of `posthoc` as its table prints it, `"low − mid"`. Without it, the figure of the whole test.

## Value
A `Float`, or `NA` for a test without an effect size. A name that is no row of the table is an
error (Z0414) that lists the rows.

## See also
statistic, estimates, ci, anova

## Examples
> let t = dataset(wages).t_test(educ, group: female)
> t.effect_size()
-0.0173753
> dataset(wages).chi2_test(union, female).effect_size()
NA
> let a = dataset(wages).add(lwage = log(wage)).anova(lwage ~ industry * category(female) + educ)
note: 211 rows with NA in a model variable dropped (line 4)
> a.effect_size()
NA
> a.effect_size("educ")
0.0803188
> a.effect_size("industry")
0.0202207
