# t_test

## Summary
t-tests for one mean, two groups or paired measurements

## In brief
- `d.t_test(wage, mu: 20)`: one mean against a value.
- `d.t_test(wage, group: union)`: the means of two groups, by Welch's test; `equal_var: true` pools the two variances.
- `d.t_test(before, after, paired: true)`: two measurements on the same rows.
- `alternative: less` or `greater` makes the test one-sided, and `level:` sets the level of the interval and of the decision.

## Description
Three tests under one name, chosen by the arguments given: a one-sample test of the mean of
`var` against `mu:`; a two-group comparison of `var` between the two levels of `group:`; or a
paired comparison of `var` and `b` row by row, with `paired: true`. Each prints the hypotheses it
tests, H0 and H1, in words and symbols; the means; the difference with its confidence interval;
t, the degrees of freedom and the p-value; in the two-group form an effect size; and a decision
at α = 1 − level beside the p-value.

The two-group form is Welch's test by default, as R's `t.test` and unlike Stata's `ttest`, which
assumes equal variances; `equal_var: true` gives the pooled test. The effect size is Hedges' g,
which corrects Cohen's d for small samples; `effect: d` gives d. Rows with `NA` in a tested
variable are dropped and counted in a note.

## Arguments
- data: The frame.
- var: The variable tested; in the paired form, the first measurement.
- b: The second measurement of the paired form, with `paired: true`; omitted otherwise.
- mu: The hypothesised mean of the one-sample form.
- group: The variable whose two groups are compared; one level or more than two is an error
  that suggests `anova` (Z0412).
- equal_var: Pool the variances (Student's test) instead of Welch's.
- effect: The effect size reported in the two-group form: Cohen's d or Hedges' g.
- paired: `true` selects the paired form, `t_test(a, b, paired: true)`.
- alternative: The alternative tested, which H1, the p-value and the interval follow: `less`
  means the first-named quantity (the mean, the first group, `var`) is less than the second or
  than `mu`.
- level: Confidence level.
- weights: Frequency or analytic weights; sampling weights are refused (Z0411) and point to
  regression with robust errors.

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`): the statistic, the degrees of freedom, the p-value, the estimate (the mean
difference) with its interval, the effect size in the two-group form, and the hypotheses. Its own
methods `statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and `alternative()` give the figures
as values.

## Missing values
A row with `NA` in the variable tested or in `group:` is left out, and a note gives the count. In the paired form a row is left out when either of its two measurements is missing. With `weights:`, a row whose weight is `NA` is dropped and counted with them, and the note names the weight; a row whose weight is zero is left out, and the note says how many.

## Details
Welch's degrees of freedom are the Welch–Satterthwaite equation. Cohen's d divides the difference
of means by the pooled standard deviation; Hedges' g = d × (1 − 3 / (4(n₁ + n₂) − 9)). The
interval of the effect size is the large-sample one of Hedges and Olkin, with standard error
√((n₁ + n₂) / (n₁ n₂) + d² / (2 (n₁ + n₂))); R's `effectsize` package gives a noncentral-t
interval by default, which differs in the third decimal for small samples. The paired test is the
one-sample test on `var − b`. p-values come from the t distribution with the tail computed
directly, never as 1 − CDF; a one-sided alternative gives a one-sided interval, as R does. Under
frequency weights N is the sum of the weights; under analytic weights the variance uses the
weights scaled to sum to n, as Stata's `aweight`.

A value above 1e150 in size cannot be squared by the arithmetic, so a variable that holds one
is refused (Z0803): divide it by a power of ten first, which changes no t and no p-value. R
prints a standard deviation of `Inf`, t = 0 and p = 1 for such values.

## A common mistake
Naming a grouping variable with more than two groups. A t-test compares two means:

> dataset(firms).t_test(size, group: industry)
error[Z0412]: `group:` of `t_test` has 12 levels, but a two-group test compares two
  = help: `anova` compares several groups and `kruskal_test` ranks them; `where` keeps two of them for this test

Compare all the groups at once with `anova(size, group: industry)`, or keep two of them first with `where`.

## Coming from R or Stata
| d.t_test(y, mu: 3) | t.test(d$y, mu = 3) | ttest y == 3 |
| d.t_test(y, group: g) | t.test(y ~ g, data = d) | ttest y, by(g) unequal |
| d.t_test(y, group: g, equal_var: true) | t.test(y ~ g, data = d, var.equal = TRUE) | ttest y, by(g) |
| d.t_test(a, b, paired: true) | t.test(d$a, d$b, paired = TRUE) | ttest a == b |
| d.t_test(y, group: g, alternative: greater, level: 0.9) | t.test(y ~ g, data = d, alternative = "greater", conf.level = 0.9) | ttest y, by(g) unequal level(90) |

Stata's `ttest` pools the two variances unless `unequal` is written; here, as in R, the
variances are kept apart unless `equal_var: true` is. Stata prints the p-values of the three
alternatives in every table, so it has no option for the alternative. Neither prints an effect
size with the test: in R they are `effectsize::hedges_g` and `effectsize::cohens_d`, in Stata
`esize twosample y, by(g)`.

## Related tasks
- To compare the means of more than two groups, use `anova`.
- To compare two groups by ranks, without supposing normal data, use `wilcoxon_test`.
- To compare two proportions, use `proportion_test`.
- To check whether two groups have the same variance, use `variance_test`.
- For a difference adjusted for other variables, use `reg`.

## References
Welch (1947), The generalization of "Student's" problem when several different population
variances are involved, Biometrika 34, 28–35.
Hedges (1981), Distribution theory for Glass's estimator of effect size and related estimators,
Journal of Educational Statistics 6, 107–128.
Hedges and Olkin (1985), Statistical Methods for Meta-Analysis, Academic Press, chapter 5.

## See also
wilcoxon_test, anova, proportion_test, reg

## Examples
// Two groups by Welch's test, the default: the log wages of men and women.
> let wages = dataset(wages).add(lwage = log(wage))
> wages.t_test(lwage, group: female)
t_test · Welch two-sample · lwage by female
  H0: the population means of lwage are equal in male and female        μ(male) − μ(female) = 0
  H1: the population means of lwage differ between male and female      μ(male) − μ(female) ≠ 0

  group        N    mean      sd
  male     9,375   3.259   0.480
  female   8,826   3.121   0.480

difference (male − female) = 0.138   95% CI [0.124, 0.152]
t = 19.35   df = 18,130.3   p < 0.001
Hedges' g = 0.29   95% CI [0.26, 0.32]
decision at α = 0.05: reject H0 (p < 0.001)
note: 211 rows with NA in lwage or female dropped (line 2)

// One mean against a value, one-sided: is the mean log wage above 3?
> wages.t_test(lwage, mu: 3, alternative: greater)
t_test · one-sample · lwage
  H0: the population mean of lwage equals 3               μ = 3
  H1: the population mean of lwage is greater than 3      μ > 3

N = 18,201   mean = 3.192   sd = 0.485
difference (mean − 3) = 0.192   95% CI [0.186, Inf)
t = 53.42   df = 18,200   p < 0.001
decision at α = 0.05: reject H0 (p < 0.001)
note: 211 rows with NA in lwage dropped (line 3)

// Two measurements on the same ten people: hours of sleep gained under two drugs.
> let sleep = frame(before = [0.7, -1.6, -0.2, -1.2, -0.1, 3.4, 3.7, 0.8, 0.0, 2.0], after = [1.9, 0.8, 1.1, 0.1, -0.1, 4.4, 5.5, 1.6, 4.6, 3.4])
> sleep.t_test(before, after, paired: true)
t_test · paired · before and after
  H0: the population mean of the paired differences before − after is 0          μ_d = 0
  H1: the population mean of the paired differences before − after is not 0      μ_d ≠ 0

N = 10 pairs   mean(before) = 0.750   mean(after) = 2.330   sd of differences = 1.230
difference (before − after) = -1.580   95% CI [-2.460, -0.700]
t = -4.06   df = 9   p = 0.003
decision at α = 0.05: reject H0 (p = 0.003)

// The pooled test, with Cohen's d for the effect and a 90% interval; the decision is then at α = 0.1.
> wages.t_test(lwage, group: female, equal_var: true, effect: d, level: 0.9)
t_test · pooled two-sample · lwage by female
  H0: the population means of lwage are equal in male and female        μ(male) − μ(female) = 0
  H1: the population means of lwage differ between male and female      μ(male) − μ(female) ≠ 0

  group        N    mean      sd
  male     9,375   3.259   0.480
  female   8,826   3.121   0.480

difference (male − female) = 0.138   90% CI [0.126, 0.149]
t = 19.35   df = 18,199   p < 0.001
Cohen's d = 0.29   90% CI [0.26, 0.31]
decision at α = 0.1: reject H0 (p < 0.001)
note: 211 rows with NA in lwage or female dropped (line 6)

// The defaults written out give the first table again.
> wages.t_test(lwage, group: female, equal_var: false, effect: g, paired: false, alternative: two_sided, level: 0.95)
t_test · Welch two-sample · lwage by female
  H0: the population means of lwage are equal in male and female        μ(male) − μ(female) = 0
  H1: the population means of lwage differ between male and female      μ(male) − μ(female) ≠ 0

  group        N    mean      sd
  male     9,375   3.259   0.480
  female   8,826   3.121   0.480

difference (male − female) = 0.138   95% CI [0.124, 0.152]
t = 19.35   df = 18,130.3   p < 0.001
Hedges' g = 0.29   95% CI [0.26, 0.32]
decision at α = 0.05: reject H0 (p < 0.001)
note: 211 rows with NA in lwage or female dropped (line 7)

// Data given as counts: each row is a score with the number of people who gave it. Frequency weights count each row that many times.
> let scores = frame(score = [1, 2, 3, 4, 5, 1, 2, 3, 4, 5], arm = ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b"], count = [3, 8, 14, 9, 4, 6, 11, 12, 5, 2])
> scores.t_test(score, group: arm, weights: count, weight_type: frequency)
t_test · Welch two-sample · score by arm
  H0: the population means of score are equal in a and b        μ(a) − μ(b) = 0
  H1: the population means of score differ between a and b      μ(a) − μ(b) ≠ 0

  group    N    mean      sd
  a       38   3.079   1.100
  b       36   2.611   1.103

difference (a − b) = 0.468   95% CI [-0.043, 0.978]
t = 1.83   df = 71.8   p = 0.072
Hedges' g = 0.42   95% CI [-0.04, 0.88]
decision at α = 0.05: fail to reject H0 (p = 0.072)

// The figures of a result as values: t, its degrees of freedom and the p-value.
> let t = scores.t_test(score, group: arm, weights: count, weight_type: frequency)
> t.statistic()
> t.df()
> t.p_values()
1.82643
71.7642
0.0719441
