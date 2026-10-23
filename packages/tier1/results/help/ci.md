# ci

## Summary
Confidence intervals of the estimates

## Description
Gives the confidence intervals of a fitted model's terms at the level asked for, computed from
the estimates, their standard errors and the fit's t table, so that any level is available after
the fit: every term with an estimate as a table of lower and upper limits, or `[lower, upper]`
when a term is named; a term omitted as collinear gives `NA`. Like `confint` in R.

## Arguments
- fit: The result.
- term: A term of the model, a bare name or text: `educ`, `industry=Retail`, `educ:female`.

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
