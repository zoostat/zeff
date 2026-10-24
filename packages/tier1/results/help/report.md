# report

## Summary
Publication table of several regressions

## Description
Sets several regressions side by side in the layout of a journal table: one column per fit,
headed `(1)`, `(2)` and so on; each term's estimate with its significance stars and its standard
error in parentheses beneath; a rule; then the kind of standard errors, the number of
observations, R² and any other statistics asked for. The table prints as a statement and
`export` writes it to Word or LaTeX with the same layout. A term a model does not have leaves its
cells empty; a term dropped as collinear shows `(omitted)`. Like `modelsummary` in R and Stata's
`esttab`.

## Arguments
- fits: The regression fits, in a list: `[m1, m2]`.
- stats: The statistics listed under the rule, from `n_obs`, `r_squared`, `aic` and `bic`;
  `[n_obs, r_squared]` unless given.
- digits: The decimals of the estimates and standard errors.
- stars: Whether to mark significance with stars and print the legend.
- labels: Text to print instead of a term's name: `labels: {educ: "Years of schooling"}`.
- title: The title printed above the table.

## Value
A `Table`, which `export` writes to `.docx` or `.tex`.

## Details
Estimates and standard errors are rounded half to even to `digits:` decimals, three unless
given, as section 3.8 of the Language Reference says; R² keeps three decimals, AIC and BIC one,
and N is a count. The stars are `*` below 0.05, `**` below 0.01 and `***` below 0.001, and
hang to the right of the estimates so that the numbers align on their decimal points. The
footer carries the legend, then every note and statistical warning a fit raised (a dropped
term, few clusters, an observation that drives a coefficient), numbered as the fit's column is.
Fixed effects and the within R² join the table with release 0.6.0.

## See also
export, reg, summary, estimates

## Examples
> let wages = dataset(wages)
> let m1 = wages.reg(log(wage) ~ educ + exper + female)
> let m2 = wages.reg(log(wage) ~ educ + exper + exper^2 + female, se: robust)
> report([m1, m2])
                    (1)         (2)
  (intercept)    1.900***    1.787***
                (0.022)     (0.023)
  educ           0.076***    0.076***
                (0.001)     (0.001)
  exper          0.022***    0.042***
                (0.000)     (0.002)
  female        -0.133***   -0.132***
                (0.006)     (0.006)
  exper^2                   -0.001***
                            (0.000)
  ───────────────────────────────────
  SEs           classical      robust
  N                18,179      18,179
  R²                0.203       0.210

* p < 0.05, ** p < 0.01, *** p < 0.001
> report([m1, m2], stats: [n_obs, r_squared, aic], labels: {educ: "Years of schooling"}, title: "Table 1").export("output/table1.docx")
