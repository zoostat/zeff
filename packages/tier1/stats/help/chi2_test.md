# chi2_test

## Summary
Chi-squared test of independence or of goodness of fit

## Description
Pearson's chi-squared test: of independence between two variables, from their cross-table, or of
goodness of fit of one variable's frequencies to the proportions `p:`, one per value in the
order the values are listed. Each result prints the hypotheses, H0 and H1; the table of counts
(observed, and for the goodness of fit the expected counts too); chi-squared with its degrees of
freedom and the p-value; and a decision at α = 0.05 beside the p-value.

No Yates correction is applied unless `correct: true`, and then on a 2 × 2 table only, as R's
`chisq.test` applies it (R corrects 2 × 2 tables by default; Stata's `tabulate, chi2` never
does). When an expected count falls below 5 the chi-squared approximation may be poor, and a
warning (W0211) says so and points to `fisher_test`, which is exact. Values are listed in the
order of section 3.7 of the Language Reference; rows with `NA` in a tested variable are dropped
and counted in a note.

## Arguments
- data: The frame.
- a: The variable tested: one of the two of a cross-table, or the one whose fit is tested.
- b: The second variable of the cross-table; omitted in the goodness-of-fit form.
- correct: Apply Yates's continuity correction to a 2 × 2 table.
- p: The proportions of the goodness-of-fit form, one per value of `a` in the listed order,
  positive and summing to 1.
- weights: Frequency weights, each row standing for that many rows; analytic weights are
  refused, since the test counts rows, and sampling weights by the shared rule (Z0411).

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`): the statistic, the degrees of freedom, the p-value and the hypotheses; it has
no estimate. Its own methods `statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and
`alternative()` give the figures as values.

## Details
Independence: expected counts E = (row total × column total) / N, χ² = Σ (|O − E| − c)² / E with
c = ½ on a corrected 2 × 2 table (never more than the smallest |O − E|) and 0 otherwise, on
(r − 1)(c − 1) degrees of freedom. Goodness of fit: E = N p, χ² = Σ (O − E)² / E on k − 1 degrees
of freedom; the proportions must sum to 1, where R's `chisq.test` rescales them only when asked.
The p-value is the upper tail of the chi-squared distribution, computed directly.

## References
Pearson (1900), On the criterion that a given system of deviations from the probable in the case
of a correlated system of variables is such that it can be reasonably supposed to have arisen
from random sampling, Philosophical Magazine 50, 157–175.
Yates (1934), Contingency tables involving small numbers and the χ² test, Supplement to the
Journal of the Royal Statistical Society 1, 217–235.

## See also
fisher_test, proportion_test, tabulate

## Examples
> let wages = dataset(wages)
> wages.chi2_test(union, female)
chi2_test · independence · union × female
  H0: union and female are independent in the population
  H1: union and female are associated in the population

  union    male   female    Total
  no      6,516    6,040   12,556
  yes     2,975    2,881    5,856
  ───────────────────────────────
  Total   9,491    8,921   18,412

chi2(1) = 1.91   p = 0.167
decision at α = 0.05: fail to reject H0 (p = 0.167)
> wages.chi2_test(female, p: [0.5, 0.5])
chi2_test · goodness of fit · female
  H0: the population proportions of female equal the given p                     π = p
  H1: at least one population proportion of female differs from the given p      π ≠ p

  female   observed   expected     p
  male        9,491    9,206.0   0.5
  female      8,921    9,206.0   0.5
  ──────────────────────────────────
  Total      18,412   18,412.0     1

chi2(1) = 17.65   p < 0.001
decision at α = 0.05: reject H0 (p < 0.001)
