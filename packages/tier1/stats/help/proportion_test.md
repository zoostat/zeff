# proportion_test

## Summary
Test of one or two proportions

## In brief
- `d.proportion_test(union, p: 0.3)`: one proportion against a value.
- `d.proportion_test(union, group: female)`: the proportions of two groups compared.
- `ci:` chooses the interval: `wilson` (the default), `wald` or, for one proportion, `exact`.
- `correct: true` applies Yates's continuity correction; `alternative:` and `level:` are as in `t_test`.

## Description
A test of one proportion against `p:`, or a comparison of two proportions between the two levels
of `group:`. The variable counted is a Bool, or a variable of 0 and 1; a success is `true` or 1.
Each result prints the hypotheses, H0 and H1, in words and symbols; the counts; the proportion,
or the difference of the two, with its confidence interval; z and the p-value; and a decision at
α = 1 − level beside the p-value.

The statistic is the uncorrected z of Stata's `prtest`, the signed square root of the chi-squared
of R's `prop.test`; `correct: true` applies Yates's continuity correction, which R applies by
default. The interval is Wilson's score interval unless `ci:` asks for Wald's or, for one
proportion, the exact Clopper–Pearson interval of R's `binom.test`; for a difference of two
proportions `wilson` is Newcombe's score interval and `exact` is refused. Rows with `NA` in a
tested variable are dropped and counted in a note.

## Arguments
- data: The frame.
- var: The Bool, or 0/1, variable whose proportion of `true` or 1 is tested.
- p: The hypothesised proportion of the one-sample form, strictly between 0 and 1.
- correct: Apply Yates's continuity correction to the statistic and the interval.
- ci: The interval: Wilson's score interval, Wald's, or the exact Clopper–Pearson interval
  (one proportion only).
- group: The variable whose two groups are compared; one level or more than two is an error
  that suggests `anova` (Z0412).
- alternative: The alternative tested, which H1, the p-value and the interval follow: `less`
  means the proportion is less than `p`, or less in the first group than in the second.
- level: Confidence level.
- weights: Frequency weights, each row standing for that many rows; analytic weights are
  refused, since a proportion is a count, and sampling weights by the shared rule (Z0411).

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`): the statistic, the p-value, the proportion or the difference with its interval,
and the hypotheses. Its own methods `statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and
`alternative()` give the figures as values.

## Missing values
A row with `NA` in the variable counted or in `group:` is left out, and a note gives the count. With `weights:`, a row whose weight is missing is left out as well.

## Details
One proportion: z = (x − n p) / √(n p (1 − p)), the correction subtracting ½ from |x − n p|
without crossing zero; Wilson's interval as `prop.test` computes it, the correction widening it
by ½ / n on each side; Wald's interval p̂ ± z √(p̂ (1 − p̂) / n); the exact interval from the
quantiles of the beta distribution. Two proportions: z is the signed square root of Pearson's
chi-squared on the 2 × 2 table of successes and failures, with the pooled proportion under H0;
Wald's interval is that of `prop.test`; Newcombe's (1998, method 10; method 11 with the
correction) combines the score limits of the two proportions. A one-sided alternative gives a
one-sided interval, bounded by 0 or 1 (−1 or 1 for a difference), as R does. p-values come from
the normal distribution with the tail computed directly, never as 1 − CDF.

## A common mistake
Giving the hypothesised proportion as a percentage:

> dataset(wages).proportion_test(union, p: 30)
error[Z0401]: `p:` of `proportion_test` is a proportion strictly between 0 and 1, but this is 30
  = help: write the hypothesised proportion, such as `p: 0.5`

Write it as a share: `p: 0.3`.

## Coming from R or Stata
| d.proportion_test(y, p: 0.3) | prop.test(sum(d$y), nrow(d), p = 0.3, correct = FALSE) | prtest y == 0.3 |
| d.proportion_test(y, p: 0.3, correct: true) | prop.test(sum(d$y), nrow(d), p = 0.3) | (none of its own) |
| d.proportion_test(y, p: 0.3, ci: exact) | binom.test(sum(d$y), nrow(d), p = 0.3), for the interval | ci proportions y, exact |
| d.proportion_test(y, group: g) | prop.test(table(d$g, d$y)[, 2:1], correct = FALSE) | prtest y, by(g) |

R's `prop.test` prints chi-squared, which is the square of the z given here, and corrects for
continuity unless `correct = FALSE` is written. Its interval for one proportion is Wilson's,
as here; for a difference it is Wald's, which is `ci: wald`. Stata's `prtest` prints z with
Wald's intervals. With `ci: exact` the interval is that of `binom.test`, while the statistic
and the p-value stay those of the z test. The R lines suppose a variable of 0 and 1 with no
missing value.

## Related tasks
- For a variable with more than two categories, use `chi2_test`.
- For an exact test of a 2 × 2 table, use `fisher_test`.
- For the same comparison of two means, use `t_test`.
- For a proportion adjusted for other variables, use `glm`.
- For the proportion in each group as a frame, use `aggregate`.

## References
Wilson (1927), Probable inference, the law of succession, and statistical inference, Journal of
the American Statistical Association 22, 209–212.
Newcombe (1998), Interval estimation for the difference between independent proportions:
comparison of eleven methods, Statistics in Medicine 17, 873–890.
Clopper and Pearson (1934), The use of confidence or fiducial limits illustrated in the case of
the binomial, Biometrika 26, 404–413.

## See also
chi2_test, fisher_test, t_test

## Examples
// One proportion against a value: is the share of union members 0.3?
> let wages = dataset(wages)
> wages.proportion_test(union, p: 0.3)
proportion_test · one-sample · union
  H0: the population proportion of union equals 0.3            π = 0.3
  H1: the population proportion of union differs from 0.3      π ≠ 0.3

N = 18,412   successes = 5,856
proportion (Wilson) = 0.318   95% CI [0.311, 0.325]
z = 5.35   p < 0.001
decision at α = 0.05: reject H0 (p < 0.001)

// Two groups: the share of union members among men and among women.
> wages.proportion_test(union, group: female)
proportion_test · two-sample · union by female
  H0: the population proportions of union are equal in male and female        π(male) − π(female) = 0
  H1: the population proportions of union differ between male and female      π(male) − π(female) ≠ 0

  group        N   successes   proportion
  male     9,491       2,975        0.313
  female   8,921       2,881        0.323

difference (male − female, Newcombe) = -0.009   95% CI [-0.023, 0.004]
z = -1.38   p = 0.167
decision at α = 0.05: fail to reject H0 (p = 0.167)

// Ten tosses of a coin, with the exact Clopper–Pearson interval.
> let coin = frame(heads = [true, true, false, true, true, true, false, true, true, false])
> coin.proportion_test(heads, p: 0.5, ci: exact)
proportion_test · one-sample · heads
  H0: the population proportion of heads equals 0.5            π = 0.5
  H1: the population proportion of heads differs from 0.5      π ≠ 0.5

N = 10   successes = 7
proportion (exact) = 0.700   95% CI [0.348, 0.933]
z = 1.26   p = 0.206
decision at α = 0.05: fail to reject H0 (p = 0.206)

// With Yates's correction, which R applies unless told not to.
> wages.proportion_test(union, p: 0.3, correct: true)
proportion_test · one-sample · union · continuity-corrected
  H0: the population proportion of union equals 0.3            π = 0.3
  H1: the population proportion of union differs from 0.3      π ≠ 0.3

N = 18,412   successes = 5,856
proportion (Wilson) = 0.318   95% CI [0.311, 0.325]
z = 5.34   p < 0.001
decision at α = 0.05: reject H0 (p < 0.001)

// One-sided, with a 90% Wald interval: is the share lower among men than among women?
> wages.proportion_test(union, group: female, alternative: less, level: 0.9, ci: wald)
proportion_test · two-sample · union by female
  H0: the population proportions of union are equal in male and female       π(male) − π(female) = 0
  H1: the population proportion of union is less in male than in female      π(male) − π(female) < 0

  group        N   successes   proportion
  male     9,491       2,975        0.313
  female   8,921       2,881        0.323

difference (male − female, Wald) = -0.009   90% CI [-1.000, -0.001]
z = -1.38   p = 0.083
decision at α = 0.1: reject H0 (p = 0.083)

// A table given as counts, one row for each cell: 18 of 25 improved with the treatment and 9 of 25 without.
> let trial = frame(treated = ["yes", "yes", "no", "no"], improved = [1, 0, 1, 0], count = [18, 7, 9, 16])
> trial.proportion_test(improved, group: treated, weights: count, weight_type: frequency)
proportion_test · two-sample · improved by treated
  H0: the population proportions of improved are equal in no and yes        π(no) − π(yes) = 0
  H1: the population proportions of improved differ between no and yes      π(no) − π(yes) ≠ 0

  group    N   successes   proportion
  no      25           9        0.360
  yes     25          18        0.720

difference (no − yes, Newcombe) = -0.360   95% CI [-0.569, -0.084]
z = -2.55   p = 0.011
decision at α = 0.05: reject H0 (p = 0.011)
