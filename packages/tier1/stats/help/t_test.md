# t_test

## Summary
t-tests for one mean, two groups or paired measurements

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

## References
Welch (1947), The generalization of "Student's" problem when several different population
variances are involved, Biometrika 34, 28–35.
Hedges (1981), Distribution theory for Glass's estimator of effect size and related estimators,
Journal of Educational Statistics 6, 107–128.
Hedges and Olkin (1985), Statistical Methods for Meta-Analysis, Academic Press, chapter 5.

## See also
wilcoxon_test, anova, proportion_test, reg

## Examples
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
> wages.t_test(lwage, mu: 3, alternative: greater)
t_test · one-sample · lwage
  H0: the population mean of lwage equals 3               μ = 3
  H1: the population mean of lwage is greater than 3      μ > 3

N = 18,201   mean = 3.192   sd = 0.485
difference (mean − 3) = 0.192   95% CI [0.186, Inf)
t = 53.42   df = 18,200   p < 0.001
decision at α = 0.05: reject H0 (p < 0.001)
note: 211 rows with NA in lwage dropped (line 3)
> let sleep = frame(before = [0.7, -1.6, -0.2, -1.2, -0.1, 3.4, 3.7, 0.8, 0.0, 2.0], after = [1.9, 0.8, 1.1, 0.1, -0.1, 4.4, 5.5, 1.6, 4.6, 3.4])
> sleep.t_test(before, after, paired: true)
t_test · paired · before and after
  H0: the population mean of the paired differences before − after is 0          μ_d = 0
  H1: the population mean of the paired differences before − after is not 0      μ_d ≠ 0

N = 10 pairs   mean(before) = 0.750   mean(after) = 2.330   sd of differences = 1.230
difference (before − after) = -1.580   95% CI [-2.460, -0.700]
t = -4.06   df = 9   p = 0.003
decision at α = 0.05: reject H0 (p = 0.003)
