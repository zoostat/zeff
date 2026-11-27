# mcnemar_test

## Summary
McNemar's test of two paired proportions

## In brief
- `d.mcnemar_test(a, b)`: whether the proportion of a value is the same in two paired variables.
- `correct: false` drops the continuity correction, and `method: exact` gives the binomial test.
- It prints the paired table and the difference of the two proportions with its interval.

## Description
McNemar's test for two variables of the same frame that hold the same two values: an answer
before and after, a diagnosis by two tests, a rating by two judges. Each row is a pair, and the
test asks whether the proportion of one value is the same in both variables. Only the pairs on
which the two differ carry information: if the proportions are equal, a pair that differs is as
likely to differ one way as the other.

`method: chi2`, the default, compares the two counts of such pairs by a chi-squared statistic on
one degree of freedom, with a continuity correction unless `correct: false`. `method: exact`
gives the exact binomial p-value of the same comparison, the one to use when few pairs differ.
Each result prints the hypotheses, H0 and H1; the paired table; the difference of the two
proportions with its 95% interval; the statistic and the p-value; a decision at α = 0.05; and
the number of pairs that differ, each way.

Values are listed in the order of section 3.7 of the Language Reference, and the proportion
spoken of is that of the second value: `yes` of `no` and `yes`, 1 of 0 and 1, `true` of `false`
and `true`. Rows with `NA` in either variable are dropped and counted in a note.

## Arguments
- data: The frame, one row for each pair.
- a: The first variable, down the side of the table.
- b: The second variable, along the top; it holds the same two values as `a`.
- correct: Whether the chi-squared statistic takes the continuity correction; `true`, the
  default, as R's `mcnemar.test` does. It has no part in `method: exact`.
- method: `chi2`, the default, for the chi-squared test, or `exact` for the binomial test of
  the pairs that differ.
- weights: Frequency weights, each row standing for that many pairs; the counts must come to
  whole numbers. Analytic weights are refused, since the test counts pairs, and sampling weights
  by the shared rule (Z0411).

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`): the p-value and the hypotheses, the difference of the two proportions with
its interval, and under `method: chi2` the statistic and its degree of freedom. Its own methods
`statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and `alternative()` give the figures as
values.

## Missing values
A row with `NA` in either variable is left out, and a note gives the count; `NA` is never a category of the table. With `weights:`, a row whose weight is `NA` is dropped and counted with them, and the note names the weight; a row whose weight is zero is left out, and the note says how many.

## Details
Write the paired table with `a` down the side and `b` along the top, and call b the number of
pairs with the second value in `b` alone and c the number with it in `a` alone. The pairs on
which the two agree enter neither test. The chi-squared statistic is (b − c)² / (b + c), and
with the continuity correction (|b − c| − 1)² / (b + c), referred to the chi-squared
distribution on one degree of freedom. Two equal counts have no difference to correct, and
give a statistic of 0 with the correction or without, as in R's `mcnemar.test`. With no pair
that differs there is no statistic, and the call is refused.

Under H0 each pair that differs falls either way with probability one half, so the smaller of
b and c is binomial on b + c trials. `method: exact` gives twice the probability of a count no
larger than the smaller one, and never more than 1; it is 1 when no pair differs.

The difference printed is the proportion of the second value in `a` less that in `b`, which
is (c − b) / n over the n pairs. Its standard error is √(b + c − (b − c)² / n) / n, and the
interval is the difference plus and minus 1.96 standard errors, kept within −1 and 1.

## A common mistake
Comparing the two proportions with `chi2_test` or `proportion_test(group:)`, which take the
two samples as independent. Here the same people answered twice, and the test must use the
pairs. Another slip is to give two variables with other values than each other's:

> let visits = frame(first = ["pos", "pos", "neg", "neg"], second = [1, 0, 1, 0])
> visits.mcnemar_test(first, second)
error[Z0401]: `mcnemar_test` compares two variables of the same two values, but `first` has neg and pos and `second` has 0 and 1
  = help: give both variables the same two values, of the same type and with the same labels

Give both variables the same two values.

## Coming from R or Stata
| d.mcnemar_test(a, b) | mcnemar.test(table(d$a, d$b)) | mcc b a |
| d.mcnemar_test(a, b, correct: false) | mcnemar.test(table(d$a, d$b), correct = FALSE) | mcc b a |
| d.mcnemar_test(a, b, method: exact) | binom.test(c(b, c)) on the two counts | mcc b a |

R's `mcnemar.test` corrects for continuity by default, as here, and has no exact test: the
binomial test of the two counts gives it. Stata's `mcc` prints the uncorrected chi-squared
and the exact probability together, with the difference of the proportions.

## Related tasks
- For two proportions from independent samples, use `proportion_test`.
- For the association of the two variables, not their difference, use `chi2_test` or `fisher_test`.
- For the paired table itself, with percentages, use `tabulate`.
- For paired measurements, not yes and no, use `t_test` or `wilcoxon_test`.

## References
McNemar (1947), Note on the sampling error of the difference between correlated proportions or
percentages, Psychometrika 12, 153–157.
Agresti (2002), Categorical Data Analysis, 2nd edition, Wiley, section 10.1.

## See also
proportion_test, chi2_test, fisher_test, tabulate

## Examples
// The same question put twice: did the share who would recommend the firm change?
> let survey = dataset(engagement)
> survey.mcnemar_test(recommend_before, recommend_after)
mcnemar_test · recommend_before × recommend_after · continuity-corrected
  H0: the population proportions of yes are equal in recommend_before and recommend_after
      π(recommend_before) − π(recommend_after) = 0
  H1: the population proportions of yes differ between recommend_before and recommend_after
      π(recommend_before) − π(recommend_after) ≠ 0

  recommend_before    no   yes   Total
  no                 465   183     648
  yes                114   415     529
  ────────────────────────────────────
  Total              579   598   1,177

difference (recommend_before − recommend_after, Wald) = -0.059   95% CI [-0.087, -0.030]
chi2(1) = 15.57   p < 0.001
decision at α = 0.05: reject H0 (p < 0.001)
297 of 1,177 pairs differ: recommend_before = yes alone in 114, recommend_after = yes alone in 183
note: 23 rows with NA in recommend_before or recommend_after dropped (line 2)

// A table given as counts, one row for each cell. Few pairs differ, so the exact test.
> let scans = frame(first = ["pos", "pos", "neg", "neg"], second = ["pos", "neg", "pos", "neg"], patients = [8, 1, 9, 12])
> scans.mcnemar_test(first, second, method: exact, weights: patients, weight_type: frequency)
mcnemar_test · exact · first × second
  H0: the population proportions of pos are equal in first and second        π(first) − π(second) = 0
  H1: the population proportions of pos differ between first and second      π(first) − π(second) ≠ 0

  first   neg   pos   Total
  neg      12     9      21
  pos       1     8       9
  ─────────────────────────
  Total    13    17      30

difference (first − second, Wald) = -0.267   95% CI [-0.450, -0.083]
p = 0.021
decision at α = 0.05: reject H0 (p = 0.021)
10 of 30 pairs differ: first = pos alone in 1, second = pos alone in 9
exact p-value from the binomial distribution of the 10 pairs that differ

// The chi-squared statistic of the same table without the continuity correction.
> scans.mcnemar_test(first, second, correct: false, method: chi2, weights: patients, weight_type: frequency)
mcnemar_test · first × second
  H0: the population proportions of pos are equal in first and second        π(first) − π(second) = 0
  H1: the population proportions of pos differ between first and second      π(first) − π(second) ≠ 0

  first   neg   pos   Total
  neg      12     9      21
  pos       1     8       9
  ─────────────────────────
  Total    13    17      30

difference (first − second, Wald) = -0.267   95% CI [-0.450, -0.083]
chi2(1) = 6.40   p = 0.011
decision at α = 0.05: reject H0 (p = 0.011)
10 of 30 pairs differ: first = pos alone in 1, second = pos alone in 9

// The figures of a result as values: the statistic, the p-value and the number of pairs.
> let r = survey.mcnemar_test(recommend_before, recommend_after, correct: true)
> print(r.statistic(), r.p_values(), r.n_obs())
note: 23 rows with NA in recommend_before or recommend_after dropped (line 6)
15.569 7.95472e-05 1177

// The interval of the difference of the two proportions.
> r.ci()
ci · 95%

  term                                                     lower    upper
  difference (recommend_before − recommend_after, Wald)   -0.087   -0.030
