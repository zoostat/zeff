# ci

## Summary
Confidence intervals of the estimates

## Description
Gives the confidence intervals of a fitted model's terms, computed from the estimates, their
standard errors and the fit's t table: every term with an estimate as a table of lower and upper
limits, or `[lower, upper]` when a term is named; a term omitted as collinear gives `NA`. The
level is the one the fit was made at, so `ci()` agrees with the summary, and `level:` written
out overrides it, so that any level is available after the fit; `predict`'s intervals follow the
same rule when they arrive. Like `confint` in R, which always uses 95% unless told otherwise.

A test's result gives the intervals it was made with: those of its estimate and its effect size,
and of each comparison of `posthoc`, at the level of the test, or `NA` when it has none. Another
level is asked of the test itself, as `t_test(level: 0.9)`.

## Arguments
- fit: The result.
- term: A term of the model, a bare name or text: `educ`, `industry=Retail`, `educ:female`.
- level: The confidence level; the fit's level unless given.

## Value
A `Table` of `term`, `lower` and `upper`, titled with the level, or a list of the two limits when
a term is named.

## See also
std_errors, p_values, estimates, summary

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.ci()
ci · 95%

  term            lower     upper
  (intercept)    1.8565    1.9431
  educ           0.0733    0.0787
  exper          0.0211    0.0229
  female        -0.1455   -0.1203
> m.ci(educ, level: 0.9)
0.0737633
0.0782532
> let m90 = dataset(wages).reg(log(wage) ~ educ + exper + female, level: 0.9)
> m90.ci(educ)
0.0737633
0.0782532
