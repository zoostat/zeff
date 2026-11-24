# chi2_test

## Summary
Chi-squared test of independence or of goodness of fit

## In brief
- `d.chi2_test(union, female)`: independence of two variables, from their cross-table.
- `d.chi2_test(party, p: [0.5, 0.3, 0.2])`: the fit of one variable's frequencies to given proportions.
- `levels:` names the categories of a fit, among them those that no row holds.
- `correct: true` applies Yates's correction to a 2 × 2 table.

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

A fit to proportions is over categories, and the data may not hold every one: a party no
respondent chose, a grade nobody was given. The values of the variable are then fewer than the
proportions. `levels:` names the categories, in the order of `p:`; a category that no row
holds has an observed count of 0 and its expected count, and takes its part in the statistic
and in the degrees of freedom.

## Arguments
- data: The frame.
- a: The variable tested: one of the two of a cross-table, or the one whose fit is tested.
- b: The second variable of the cross-table; omitted in the goodness-of-fit form.
- correct: Apply Yates's continuity correction to a 2 × 2 table.
- p: The proportions of the goodness-of-fit form, one per value of `a` in the listed order,
  positive and summing to 1.
- weights: Frequency weights, each row standing for that many rows; analytic weights are
  refused, since the test counts rows, and sampling weights by the shared rule (Z0411).
- levels: In the goodness-of-fit form, the categories the proportions are for, in the order of
  `p:`: the values the variable holds and those it could hold. Every value the rows hold must
  be among them; left out, the categories are the values the rows hold.

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`): the statistic, the degrees of freedom, the p-value and the hypotheses; it has
no estimate. Its own methods `statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and
`alternative()` give the figures as values.

## Missing values
A row with `NA` in a variable tested is left out, and a note gives the count; `NA` is never a category of the table. With `weights:`, a row whose weight is missing is left out as well.

## Details
Independence: expected counts E = (row total × column total) / N, χ² = Σ (|O − E| − c)² / E with
c = ½ on a corrected 2 × 2 table (never more than the smallest |O − E|) and 0 otherwise, on
(r − 1)(c − 1) degrees of freedom. Goodness of fit: E = N p, χ² = Σ (O − E)² / E on k − 1 degrees
of freedom, k being the number of categories: the values of the variable, or those `levels:`
names, a category never observed having O = 0. In R the same test is
`chisq.test(table(factor(x, levels = lev)), p = p)`. The proportions must sum to 1, where R's
`chisq.test` rescales them only when asked.
The p-value is the upper tail of the chi-squared distribution, computed directly.

## A common mistake
Giving the proportions of a fit as percentages. They are shares that sum to one:

> dataset(wages).chi2_test(female, p: [50, 50])
error[Z0401]: `p:` of `chi2_test` must be positive proportions that sum to 1, but these sum to 100
  = help: divide each by their total, or write them as shares such as `p: [0.5, 0.3, 0.2]`

Write `p: [0.5, 0.5]`.

## Coming from R or Stata
| d.chi2_test(a, b) | chisq.test(table(d$a, d$b), correct = FALSE) | tabulate a b, chi2 |
| d.chi2_test(a, b, correct: true) | chisq.test(table(d$a, d$b)) | (none of its own) |
| d.chi2_test(a, p: [0.5, 0.3, 0.2]) | chisq.test(table(d$a), p = c(0.5, 0.3, 0.2)) | (none of its own) |
| d.chi2_test(a, p: p, levels: lev) | chisq.test(table(factor(d$a, levels = lev)), p = p) | (none of its own) |

R corrects a 2 × 2 table for continuity unless `correct = FALSE` is written; here, as in Stata,
there is no correction unless it is asked for. Stata's `tabulate, chi2` is the test of
independence of a two-way table.

## Related tasks
- For an exact test when expected counts are small, use `fisher_test`.
- For one proportion against a value, or two proportions with an interval for their difference, use `proportion_test`.
- For the table itself, with percentages, use `tabulate`.
- For a binary outcome on several variables at once, use `glm`.

## References
Pearson (1900), On the criterion that a given system of deviations from the probable in the case
of a correlated system of variables is such that it can be reasonably supposed to have arisen
from random sampling, Philosophical Magazine 50, 157–175.
Yates (1934), Contingency tables involving small numbers and the χ² test, Supplement to the
Journal of the Royal Statistical Society 1, 217–235.

## See also
fisher_test, proportion_test, tabulate

## Examples
// Independence of two variables: is union membership the same among men and women?
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

// The fit of one variable to given proportions: are men and women equally many?
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

// A category that no row holds, named with `levels:`: three parties, of which one got no vote.
> let votes = frame(party = ["a", "a", "b", "a", "b", "a", "a", "b", "a", "a"])
> votes.chi2_test(party, p: [0.5, 0.3, 0.2], levels: ["a", "b", "c"])
chi2_test · goodness of fit · party
  H0: the population proportions of party equal the given p                     π = p
  H1: at least one population proportion of party differs from the given p      π ≠ p

  party   observed   expected     p
  a              7        5.0   0.5
  b              3        3.0   0.3
  c              0        2.0   0.2
  ─────────────────────────────────
  Total         10       10.0     1

chi2(2) = 2.80   p = 0.247
decision at α = 0.05: fail to reject H0 (p = 0.247)
warning[W0211]: 2 of 3 expected counts are below 5, so the chi-squared approximation may be poor (line 5)
  = help: `fisher_test` is exact for a small table; merging sparse values raises the expected counts

// A table given as counts, one row for each cell, with frequency weights.
> let tab = frame(treated = ["yes", "yes", "no", "no"], improved = ["yes", "no", "yes", "no"], count = [18, 7, 9, 16])
> tab.chi2_test(treated, improved, weights: count, weight_type: frequency)
chi2_test · independence · treated × improved
  H0: treated and improved are independent in the population
  H1: treated and improved are associated in the population

  treated   no   yes   Total
  no        16     9      25
  yes        7    18      25
  ──────────────────────────
  Total     23    27      50

chi2(1) = 6.52   p = 0.011
decision at α = 0.05: reject H0 (p = 0.011)

// The same with Yates's correction, as R applies it to a 2 × 2 table unless told not to.
> tab.chi2_test(treated, improved, correct: true, weights: count, weight_type: frequency)
chi2_test · independence · treated × improved · continuity-corrected
  H0: treated and improved are independent in the population
  H1: treated and improved are associated in the population

  treated   no   yes   Total
  no        16     9      25
  yes        7    18      25
  ──────────────────────────
  Total     23    27      50

chi2(1) = 5.15   p = 0.023
decision at α = 0.05: reject H0 (p = 0.023)
