# estimates

## Summary
The estimates of a result as a frame

## Description
Gives the terms of a result with their estimate, standard error, statistic, p-value and
confidence interval at the result's level as a frame, one row per term with an estimate, for
tables and plots of one's own; a term omitted as collinear has no row. A test's result gives one
row with its statistic and p-value. Like `broom::tidy` in R and `r(table)` in Stata.

## Arguments
- fit: The result.

## Value
A `Frame` with the variables `term`, `estimate`, `std_error`, `statistic`, `p_value`, `ci_lower`
and `ci_upper`.

## See also
summary, std_errors, p_values, ci

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.estimates()
frame · 4 rows × 7 variables

      term           estimate     std_error   statistic       p_value    ci_lower    ci_upper
  1   (intercept)     1.89982     0.0220778     86.0514             0     1.85655      1.9431
  2   educ          0.0760082    0.00136477     55.6929             0   0.0733332   0.0786833
  3   exper           0.02202   0.000451384     48.7833             0   0.0211352   0.0229048
  4   female        -0.132924    0.00642367    -20.6929   4.86824e-94   -0.145515   -0.120333
