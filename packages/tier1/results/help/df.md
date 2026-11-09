# df

## Summary
The degrees of freedom of a test

## Description
Gives the degrees of freedom of a test's statistic as a list of numbers in the order the
statistic writes them: one for a t, a chi-squared or an H, two for an F, `F(1, 18,179)`. A
statistic without degrees of freedom, such as a z, or a test without a statistic gives `NA`.
Like `t$parameter` in R and `r(df)` or `r(df_r)` in Stata.

## Arguments
- t: A test's result.

## Value
A `List<Float>` of one or two numbers, or `NA`.

## See also
statistic, p_values, estimates

## Examples
> let t = dataset(wages).t_test(educ, group: female)
> t.df()
18304
> let m = dataset(wages).reg(educ ~ female + age)
> m.test(female == 0).df()
1
18409
