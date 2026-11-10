# posthoc

## Summary
Pairwise comparisons after an ANOVA

## Description
The comparison of every pair of group means after a one-way `anova`, written on its result:
`d.anova(y, group: g).posthoc()`. Each row is one pair of groups, the first minus the second,
with the difference of the two means, its standard error, its t statistic and a p-value that
allows for the number of comparisons made. The groups, their means and their variances are
those of the `anova`, so the rows dropped for `NA`, the weights and the order of the groups are
the same, and nothing is read from the frame again.

`method:` chooses among ten methods. `tukey`, the default, is Tukey's honestly significant
difference, in the Tukey–Kramer form when the groups differ in size, and `scheffe` is Scheffé's
method; both rest on the pooled within-group variance. The other eight take the t-test of each
pair and adjust its p-value as `p_adjust` does: `bonferroni`, `holm`, `hochberg`, `hommel` and
`sidak` for the chance of any false rejection, `bh` and `by` for the false discovery rate, and
`none` for no adjustment. Their t-tests follow the `anova`: Welch's test of each pair after
Welch's F, which is the default of `anova`, and the test on the pooled variance after
`equal_var: true`.

`tukey` and `scheffe` pool the variance whichever F the `anova` computed, as R and Stata do.
After Welch's `anova`, which did not assume equal variances, a note says so and names
`method: holm`, which compares the pairs by Welch's t-tests.

Intervals for the differences, at 95%, come with the single-step methods: `tukey`, `scheffe`,
`bonferroni`, `sidak` and `none`. The step-wise methods, `holm`, `hochberg`, `hommel`, `bh` and
`by`, adjust each p-value by its place among the others and have no intervals to match, so
their table has none.

`posthoc` follows the one-way form of `anova`. The result of an `anova` given a formula, or of
any other test, is an error (Z0401).

## Arguments
- fit: The result of a one-way `anova`, `d.anova(y, group: g)`, with or without `equal_var:`
  and `weights:`.
- method: `tukey` and `scheffe` control the chance of any false rejection on the pooled
  variance: Tukey's over the pairs, Scheffé's over every contrast of the means, so it is the
  more cautious of the two for pairs. `bonferroni`, `holm`, `hochberg`, `hommel`, `sidak`, `bh`,
  `by` and `none` adjust the p-values of the pairwise t-tests and are described under
  `p_adjust`; `holm` is never less powerful than `bonferroni`.

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`). Its table has a row for each pair of the groups of the `anova`, in the
order (1, 2), (1, 3), …, (2, 3), …: the comparison, the difference, its standard error, t, the
degrees of freedom when each pair has its own (Welch's tests), the p-value under the method,
and the interval when the method has one. `estimates()` gives the same rows as a frame, with
the difference in `estimate`, t in `statistic`, the p-value under the method in `p_value` and
the limits in `ci_lower` and `ci_upper`, which are `NA` for a step-wise method; `ci()` gives
the intervals as a table, or `NA` when the method has none. `statistic()`, `df()` and
`p_values()` give the F of the `anova` followed, its two degrees of freedom and its p-value,
and `n_obs()` the number of observations. A set of comparisons has no single hypothesis or
effect size, so `h0()`, `h1()` and `effect_size()` are `NA` and no decision line is printed.

## Details
With k groups of sizes nᵢ, means mᵢ and variances sᵢ², N = Σ nᵢ observations and
c = k(k − 1)/2 comparisons, the comparison of groups i and j has the difference d = mᵢ − mⱼ.
On the pooled variance s² = Σ (nᵢ − 1)sᵢ² / (N − k), its standard error is
se = s·√(1/nᵢ + 1/nⱼ) and t = d/se has N − k degrees of freedom. Welch's comparison has
se = √(sᵢ²/nᵢ + sⱼ²/nⱼ) and the Welch–Satterthwaite degrees of freedom of the pair, so it is
`t_test` of the two groups alone, and its p-value before adjustment is that test's.

Tukey. The p-value is P(Q > |t|·√2), where Q is the studentized range of k means on N − k
degrees of freedom (`dist.tukey`), and the interval is d ± q·se/√2, with q the 95% quantile of
Q. With groups of one size the intervals hold jointly at exactly 95%; with unequal sizes this is
the Tukey–Kramer form, which is conservative (Hayter 1984).

Scheffé. The p-value is P(F > t²/(k − 1)), with F on k − 1 and N − k degrees of freedom, and
the interval is d ± se·√((k − 1)·f), with f the 95% quantile of that F. The level holds jointly
over every contrast of the group means, the pairs among them, so for the pairs alone the
intervals are wider than Tukey's. A pair is significant by Scheffé's method only if the
classical F of the groups is.

Adjusted t-tests. With p the two-sided p-value of a pair's t-test, Bonferroni's p-value is c·p
and its interval d ± t₁·se, with t₁ the quantile of t at 1 − 0.05/(2c). Šidák's p-value is
1 − (1 − p)^c and its interval takes the quantile at 1 − (1 − 0.95^(1/c))/2. `none` gives p
and the quantile at 0.975, which after Welch's `anova` is the interval of `t_test`. Holm,
Hochberg, Hommel, Benjamini–Hochberg and Benjamini–Yekutieli are computed over the c p-values
as in `p_adjust`. No p-value is above 1.

With two groups there is one comparison, and every method gives the p-value of the t-test of
the two groups: on the pooled variance for `tukey` and `scheffe`, and as the `anova` was made
for the others.

After an `anova` with `weights:` the comparisons use its weighted sizes, means and variances.
After `equal_var: true` a group of a single value is compared with the others on the pooled
variance; Welch's comparisons need two values with some variation in every group, which
Welch's `anova` requires already.

In R, `tukey` is `TukeyHSD(aov(y ~ g))` and `scheffe` is `DescTools::ScheffeTest`. The adjusted
t-tests are `pairwise.t.test(y, g, p.adjust.method = , pool.sd = )`, with `pool.sd = TRUE`
after `equal_var: true` and `FALSE` after Welch's `anova`; R's own defaults there are `"holm"`
and the pooled variance. `sidak` on the pooled variance is `pairs(emmeans(fit, ~ g),
adjust = "sidak")`. In Stata, `pwmean y, over(g) mcompare(tukey) effects` and `pwcompare` after
`anova` give `tukey`, `scheffe`, `bonferroni`, `sidak` and, as `noadjust`, `none`, always on the
pooled variance, and `oneway y g, bonferroni scheffe sidak` prints the same p-values.
`TukeyHSD`, `ScheffeTest` and Stata write each comparison as the second group minus the first,
so their differences and limits have the opposite sign. Stata's `snk`, `duncan` and `dunnett`
and the Games–Howell comparison are not given.

The differences, standard errors, t statistics, and the p-values and limits of every method but
Tukey's agree with R to nine significant digits. Tukey's come from `dist.tukey`, which is
accurate to a relative 1e-12, while R's `ptukey` and `qtukey` are less so (the page of
`dist.tukey` gives the figures): R's Tukey p-values agree with these to six decimals on 20 or
more degrees of freedom and to four below, and R's limits to a thousandth of the standard
error.

## References
Tukey (1949), Comparing individual means in the analysis of variance, Biometrics 5, 99–114.
Kramer (1956), Extension of multiple range tests to group means with unequal numbers of
replications, Biometrics 12, 307–310.
Hayter (1984), A proof of the conjecture that the Tukey–Kramer multiple comparisons procedure
is conservative, Annals of Statistics 12, 61–75.
Scheffé (1953), A method for judging all contrasts in the analysis of variance, Biometrika 40,
87–104.
Dunn (1961), Multiple comparisons among means, Journal of the American Statistical Association
56, 52–64.
Šidák (1967), Rectangular confidence regions for the means of multivariate normal
distributions, Journal of the American Statistical Association 62, 626–633.
Welch (1947), The generalization of 'Student's' problem when several different population
variances are involved, Biometrika 34, 28–35.
Hsu (1996), Multiple Comparisons: Theory and Methods, Chapman and Hall.
The step-wise methods have their references on the page of `p_adjust`.

## See also
anova, p_adjust, t_test, dist.tukey, estimates, ci

## Examples
> let d = frame(gain = [4.2, 3.9, 4.6, 4.1, 4.4, 5.1, 5.6, 4.9, 5.4, 5.8, 4.5, 4.0, 4.8, 4.3], diet = ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b", "c", "c", "c", "c"])
> d.anova(gain, group: diet, equal_var: true).posthoc()
posthoc · Tukey HSD · gain by diet
each row tests H0: the two population means are equal, against H1: they differ

  comparison   difference   std.error   t value   p value   95% CI
  a − b            -1.120       0.206     -5.44     0.001   [-1.676, -0.564]
  a − c            -0.160       0.218     -0.73     0.750   [-0.749, 0.429]
  b − c             0.960       0.218      4.40     0.003   [0.371, 1.549]

Tukey's HSD over 3 comparisons of 3 groups: p-values and intervals from the studentized range, family-wise
pooled within-group variance on 11 degrees of freedom; N = 14
> d.anova(gain, group: diet).posthoc(method: holm)
posthoc · Holm · gain by diet
each row tests H0: the two population means are equal, against H1: they differ

  comparison   difference   std.error   t value    df   p value
  a − b            -1.120       0.203     -5.52   7.4     0.002
  a − c            -0.160       0.207     -0.77   5.7     0.471
  b − c             0.960       0.234      4.10   6.8     0.010

Holm over 3 comparisons of 3 groups: p-values adjusted family-wise; step-wise, so no intervals
Welch's t-test of each pair, the variances not assumed equal; N = 14
> d.anova(gain, group: diet).posthoc()
posthoc · Tukey HSD · gain by diet
each row tests H0: the two population means are equal, against H1: they differ

  comparison   difference   std.error   t value   p value   95% CI
  a − b            -1.120       0.206     -5.44     0.001   [-1.676, -0.564]
  a − c            -0.160       0.218     -0.73     0.750   [-0.749, 0.429]
  b − c             0.960       0.218      4.40     0.003   [0.371, 1.549]

Tukey's HSD over 3 comparisons of 3 groups: p-values and intervals from the studentized range, family-wise
pooled within-group variance on 11 degrees of freedom; N = 14
note: `tukey` pools the variances of the groups, which the `anova` did not assume equal; `method: holm` compares the pairs by Welch's t-tests (line 4)
> let b = d.anova(gain, group: diet, equal_var: true).posthoc(method: bonferroni)
> b.estimates()
frame · 3 rows × 9 variables

      term    estimate   std_error   statistic   df   df2       p_value    ci_lower    ci_upper
  1   a − b      -1.12    0.205736    -5.44387   11    NA   0.000608127    -1.70018   -0.539818
  2   a − c      -0.16    0.218216   -0.733219   11    NA             1   -0.775376    0.455376
  3   b − c       0.96    0.218216     4.39931   11    NA    0.00319324    0.344624     1.57538
> b.ci()
ci · 95%

  term     lower    upper
  a − b   -1.700   -0.540
  a − c   -0.775    0.455
  b − c    0.345    1.575
