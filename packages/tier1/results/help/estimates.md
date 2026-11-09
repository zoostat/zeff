# estimates

## Summary
The estimates of a result as a frame

## Description
Gives the terms of a result with their estimate, standard error, statistic, degrees of freedom,
p-value and confidence interval at the result's level as a frame, one row per term with an
estimate, for tables and plots of one's own; a term omitted as collinear has no row. A test's
result gives a row for its estimate, with the statistic, its degrees of freedom and the p-value,
and a row for its effect size with its interval where it has one; `df2` holds the second degrees
of freedom of an F and is `NA` otherwise. Like `broom::tidy` in R and `r(table)` in Stata.

## Arguments
- fit: The result.

## Value
A `Frame` with the variables `term`, `estimate`, `std_error`, `statistic`, `df`, `df2`,
`p_value`, `ci_lower` and `ci_upper`.

## See also
summary, std_errors, p_values, ci

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.estimates()
frame · 4 rows × 9 variables

      term           estimate     std_error   statistic      df   df2       p_value    ci_lower    ci_upper
  1   (intercept)     1.89982     0.0220778     86.0514   18175    NA             0     1.85655      1.9431
  2   educ          0.0760082    0.00136477     55.6929   18175    NA             0   0.0733332   0.0786833
  3   exper           0.02202   0.000451384     48.7833   18175    NA             0   0.0211352   0.0229048
  4   female        -0.132924    0.00642367    -20.6929   18175    NA   4.86824e-94   -0.145515   -0.120333
> dataset(wages).t_test(educ, group: female).estimates()
frame · 2 rows × 9 variables

      term                           estimate   std_error   statistic      df   df2    p_value     ci_lower
  1   difference (male − female)   -0.0431739          NA     -1.1778   18304    NA   0.238892    -0.115024
  2   Hedges' g                    -0.0173753          NA          NA      NA    NA         NA   -0.0462772

       ci_upper
  1    0.028676
  2   0.0115266
