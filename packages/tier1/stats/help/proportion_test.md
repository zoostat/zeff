# proportion_test

## Summary
Test of one or two proportions

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
`export`, `report`): the statistic, the p-value, the proportion or the difference with its
interval, and the hypotheses.

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
> let wages = dataset(wages)
> wages.proportion_test(union, p: 0.3)
proportion_test · one-sample · union
  H0: the population proportion of union equals 0.3            π = 0.3
  H1: the population proportion of union differs from 0.3      π ≠ 0.3

N = 18,412   successes = 5,856
proportion (Wilson) = 0.318   95% CI [0.311, 0.325]
z = 5.35   p < 0.001
decision at α = 0.05: reject H0 (p < 0.001)
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
> let coin = frame(heads = [true, true, false, true, true, true, false, true, true, false])
> coin.proportion_test(heads, p: 0.5, ci: exact)
proportion_test · one-sample · heads
  H0: the population proportion of heads equals 0.5            π = 0.5
  H1: the population proportion of heads differs from 0.5      π ≠ 0.5

N = 10   successes = 7
proportion (exact) = 0.700   95% CI [0.348, 0.933]
z = 1.26   p = 0.206
decision at α = 0.05: fail to reject H0 (p = 0.206)
