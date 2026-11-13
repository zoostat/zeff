# df

## Summary
The degrees of freedom of a test

## Description
Gives the degrees of freedom of a test's statistic as a list of numbers in the order the
statistic writes them: one for a t, a chi-squared or an H, two for an F, `F(1, 18,179)`. A
statistic without degrees of freedom, such as a z, or a test without a statistic gives `NA`.
Where the result prints a table of several rows, the degrees of freedom of one row are read by
naming it: the two of the F of a term of a factorial `anova`, `df("industry")`, or the one of
the t of a comparison of `posthoc`, named as its table prints it; without a name they are those
of the whole test. Like `t$parameter` in R and `r(df)` or `r(df_r)` in Stata.

## Arguments
- t: A test's result.
- term: A row of the result's table, a bare name or text: a term of a factorial `anova`, `dose` or `"dose:diet"`, or a comparison of `posthoc` as its table prints it, `"low − mid"`. Without it, the figure of the whole test.

## Value
A `List<Float>` of one or two numbers, or `NA`. A name that is no row of the table is an error
(Z0414) that lists the rows.

## See also
statistic, p_values, estimates, anova, posthoc

## Examples
> let t = dataset(wages).t_test(educ, group: female)
> t.df()
18304
> let m = dataset(wages).reg(educ ~ female + age)
> m.test(female == 0).df()
1
18409
> let a = dataset(wages).add(lwage = log(wage)).anova(lwage ~ industry * category(female) + educ)
note: 211 rows with NA in a model variable dropped (line 5)
> a.df()
24
18176
> a.df("industry:female")
11
18176
