# posthoc

## Summary
Pairwise comparisons after an ANOVA

## Description
The comparison of every pair of means after an `anova`, written on its result. Each row is one
pair, the first minus the second, with the difference of the two means, its standard error, its
t statistic and a p-value that allows for the number of comparisons made. Everything comes from
the `anova`, so the rows dropped for `NA`, the weights and the order of the levels are the same,
and nothing is read from the frame again.

After the one-way form, `d.anova(y, group: g).posthoc()`, the pairs are of the group means.

After a formula, `d.anova(y ~ a * b + x).posthoc(term: a)`, the pairs are of the estimated
marginal means of the term that `term:` names. The marginal mean of a level of `a` is what the
model predicts for that level, averaged with equal weight over the levels of every other
category of the model, each numeric variable held at its mean. It answers "what would the mean
of this level be if the other factors were balanced", which the plain mean of an unbalanced
table does not. `term: "a:b"`, in quotes, names an interaction and compares every pair of its
cells. `term:` may be left out when the model has one term made of categories.

`method:` chooses among ten methods. `tukey`, the default, is Tukey's honestly significant
difference, in the Tukey–Kramer form when the groups differ in size, and `scheffe` is Scheffé's
method; both rest on one pooled variance. The other eight take the t-test of each pair and
adjust its p-value as `p_adjust` does: `bonferroni`, `holm`, `hochberg`, `hommel` and `sidak`
for the chance of any false rejection, `bh` and `by` for the false discovery rate, and `none`
for no adjustment. After the one-way form their t-tests follow the `anova`: Welch's test of
each pair after Welch's F, which is the default of `anova`, and the test on the pooled variance
after `equal_var: true`. After a formula every method uses the residual variance of the model
and its degrees of freedom, and the family is the pairs of the one term compared.

`tukey` and `scheffe` pool the variance whichever F the `anova` computed, as R and Stata do.
After Welch's `anova`, which did not assume equal variances, a note says so and names
`method: holm`, which compares the pairs by Welch's t-tests.

Intervals for the differences come with the single-step methods: `tukey`, `scheffe`,
`bonferroni`, `sidak` and `none`. They are at the level `level:` asks for, 95% unless given, as
R's `TukeyHSD(conf.level = )` and Stata's `pwcompare, level(#)` take it; the p-values do not
depend on the level. The step-wise methods, `holm`, `hochberg`, `hommel`, `bh` and `by`, adjust
each p-value by its place among the others and have no intervals to match, so their table has
none, and `level:` written out with one of them changes nothing, which a note says (W0401).

`posthoc` follows an `anova`. The result of any other test is an error (Z0401).

## Arguments
- fit: The result of an `anova`: the one-way form, `d.anova(y, group: g)`, with or without
  `equal_var:` and `weights:`, or the form with a formula, `d.anova(y ~ a * b)`, of any `type:`.
- method: `tukey` and `scheffe` control the chance of any false rejection on the pooled
  variance: Tukey's over the pairs, Scheffé's over every contrast of the means, so it is the
  more cautious of the two for pairs. `bonferroni`, `holm`, `hochberg`, `hommel`, `sidak`, `bh`,
  `by` and `none` adjust the p-values of the pairwise t-tests and are described under
  `p_adjust`; `holm` is never less powerful than `bonferroni`.
- term: After a formula, the term whose levels are compared: a category of the model,
  `term: dose`, or an interaction of categories in quotes, `term: "dose:diet"`, whose parts may
  come in either order and set the order of the cells. It must be a term of the formula made
  of categories alone; a name that is no term is an error (Z0414) that lists the terms. It may
  be left out when the formula has one such term and is required when it has several (Z0401).
  After the one-way form there is nothing to choose and `term:` is an error (Z0401).
- level: The confidence level of the intervals, between 0 and 1; 0.95 unless given. It is the
  level that holds jointly over the comparisons under `tukey`, `scheffe`, `bonferroni` and
  `sidak`, and of each interval on its own under `none`.

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`). Its table has a row for each pair of the groups of the `anova`, or of the
levels or cells of the term, in the order (1, 2), (1, 3), …, (2, 3), …: the comparison, the
difference, its standard error, t, the degrees of freedom when each pair has its own (Welch's
tests), the p-value under the method, and the interval when the method has one. A cell of an
interaction is named by its levels joined by a colon, `low:a`, and a level that is a number
carries the name of its category, `lot=2`. Under the table of a formula's comparisons a line
says what each mean averages over and the value at which each numeric part is held.
`estimates()` gives the same rows as a frame, with
the difference in `estimate`, t in `statistic`, the p-value under the method in `p_value` and
the limits in `ci_lower` and `ci_upper`, which are `NA` for a step-wise method; `ci()` gives
the intervals as a table, or `NA` when the method has none. `statistic()`, `df()` and
`p_values()` give the F of the `anova` followed, its two degrees of freedom and its p-value
(after a formula, the F of the term compared, as the table of the `anova` has it), and
`n_obs()` the number of observations. One comparison is read by naming it as the table prints
it: `p_values("a − b")`, `statistic("a − b")` and `df("a − b")` give its p-value under the
method, its t and its degrees of freedom; the minus sign may be typed as a hyphen, `"a - b"`,
and a name that is no row of the table is an error (Z0414) that lists the rows. A set of
comparisons has no single hypothesis or effect size, so `h0()`, `h1()` and `effect_size()`
are `NA` and no decision line is printed.

## Details
With k groups of sizes nᵢ, means mᵢ and variances sᵢ², N = Σ nᵢ observations and
c = k(k − 1)/2 comparisons, the comparison of groups i and j has the difference d = mᵢ − mⱼ.
On the pooled variance s² = Σ (nᵢ − 1)sᵢ² / (N − k), its standard error is
se = s·√(1/nᵢ + 1/nⱼ) and t = d/se has N − k degrees of freedom. Welch's comparison has
se = √(sᵢ²/nᵢ + sⱼ²/nⱼ) and the Welch–Satterthwaite degrees of freedom of the pair, so it is
`t_test` of the two groups alone, and its p-value before adjustment is that test's.

Tukey. The p-value is P(Q > |t|·√2), where Q is the studentized range of k means on N − k
degrees of freedom (`dist.tukey`), and the interval is d ± q·se/√2, with q the quantile of Q at
the level, 95% unless `level:` says otherwise. With groups of one size the intervals hold
jointly at exactly the level; with unequal sizes this is the Tukey–Kramer form, which is
conservative (Hayter 1984).

Scheffé. The p-value is P(F > t²/(k − 1)), with F on k − 1 and N − k degrees of freedom, and
the interval is d ± se·√((k − 1)·f), with f the quantile of that F at the level. The level
holds jointly
over every contrast of the group means, the pairs among them, so for the pairs alone the
intervals are wider than Tukey's. A pair is significant by Scheffé's method only if the
classical F of the groups is.

Adjusted t-tests. With p the two-sided p-value of a pair's t-test, Bonferroni's p-value is c·p
and its interval d ± t₁·se, with t₁ the quantile of t at 1 − α/(2c), α being 1 less the level,
0.05 unless `level:` says otherwise. Šidák's p-value is 1 − (1 − p)^c and its interval takes
the quantile at 1 − (1 − (1 − α)^(1/c))/2. `none` gives p and the quantile at 1 − α/2, which
after Welch's `anova` is the interval of `t_test` at that level. Holm,
Hochberg, Hommel, Benjamini–Hochberg and Benjamini–Yekutieli are computed over the c p-values
as in `p_adjust`. No p-value is above 1.

With two groups there is one comparison, and every method gives the p-value of the t-test of
the two groups: on the pooled variance for `tukey` and `scheffe`, and as the `anova` was made
for the others.

After an `anova` with `weights:` the comparisons use its weighted sizes, means and variances.
After `equal_var: true` a group of a single value is compared with the others on the pooled
variance; Welch's comparisons need two values with some variation in every group, which
Welch's `anova` requires already.

After a formula. Write the fitted model as y = Xb + e, with one column of X for the intercept,
for each numeric part and for each contrast of a category and their products. A marginal mean
is a linear function m = l'b of the coefficients. The weight l gives a column is the product of
what its parts contribute: a numeric part its held value, a category of the term the value the
column takes at the level asked for, and any other category the average of the column over its
levels. The difference of two marginal means is d = (l₁ − l₂)'b, its standard error is
se = s·√((l₁ − l₂)'(X'X)⁻¹(l₁ − l₂)) with s² the residual mean square, and t = d/se has the
residual degrees of freedom of the model. The methods then apply as above with k the number of
means compared and c = k(k − 1)/2: Tukey's range is of k means, Scheffé's F has k − 1 numerator
degrees of freedom, and the adjusted methods work on the c p-values of the t-tests. The
comparisons do not depend on `type:`, which changes the table of the `anova` alone, nor on the
order of the terms in the formula.

A numeric variable is held at its mean over the rows the model used, by their weights when the
`anova` was weighted, and a part computed from it takes its value there: `log(income)` is the
logarithm of the mean income, not the mean of the logarithms. The line under the table gives
each value. This matters only when the numeric part interacts with the term compared, as in
`y ~ a * x`; otherwise it adds the same amount to every mean and leaves every difference as it
is. A `Bool` in a formula is a number, held at the share of its rows that are true.

When the term compared is also part of a larger term, a note says what its differences then
are: averages over the levels of the other category, which may hide differences that go in
opposite directions at different levels, or differences at one value of a numeric variable.
Comparing the cells, `term: "a:b"`, shows them level by level.

A combination of levels that no row holds leaves some means undetermined when the model has the
interaction of those categories: the cell itself, and the marginal mean of each level that
meets it. Such a mean is left out with a note, the comparisons are among the others, and k
counts those alone. With fewer than two left there is no pair and `posthoc` stops (Z0802). The
model without the interaction determines every mean.

With one category and no other term, `d.anova(y ~ g).posthoc()` gives the comparisons of
`d.anova(y, group: g, equal_var: true).posthoc()`. On a balanced table with no numeric part the
marginal means are the plain means of the levels. With two levels, the square of the one t is
the F of the term under `type: 3`.

In R the comparisons after a formula are `pairs(emmeans(lm(y ~ a * b + x), ~ a), adjust = )`
of the package emmeans, with `~ a:b` for the cells, and `confint()` of the same for the
intervals; emmeans writes `"BH"` and `"BY"` for `bh` and `by`. In Stata they are
`pwcompare a, mcompare(tukey) effects` after `anova y a##b c.x`, with `pwcompare a#b` for the
cells. Three differences from emmeans are deliberate. Under analytic weights emmeans holds a
numeric variable at its unweighted mean and Zeff at its weighted mean, so that frequency
weights give what the rows written out would give. emmeans treats a logical variable as a
category, and Zeff a `Bool` as a number. emmeans lists a mean it cannot estimate with empty
figures, and Zeff leaves it out with a note; both count the family without it. R's
`TukeyHSD(aov(y ~ a * b), "a")` is another quantity on an unbalanced table: it compares means
adjusted for the terms written before `a` alone, so its answer changes with the order of the
terms. On a balanced table with no numeric part the two agree.

In R, `tukey` after the one-way form is `TukeyHSD(aov(y ~ g))` and `scheffe` is
`DescTools::ScheffeTest`. The adjusted
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
Tukey's agree with R to nine significant digits, after either form of `anova`. Tukey's come
from `dist.tukey`, which is
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
Searle, Speed and Milliken (1980), Population marginal means in the linear model: an
alternative to least squares means, The American Statistician 34, 216–221.
Lenth (2016), Least-squares means: the R package lsmeans, Journal of Statistical Software
69(1), 1–33.
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
> let f = frame(gain = [4.1, 4.9, 3.6, 4.6, 5.3, 4.0, 5.0, 5.8, 4.4, 5.2, 6.4, 5.1, 5.2, 5.9, 4.6, 6.9, 7.8, 6.1], dose = ["low", "low", "low", "low", "low", "low", "mid", "mid", "mid", "mid", "mid", "mid", "top", "top", "top", "top", "top", "top"], diet = ["a", "a", "a", "b", "b", "b", "a", "a", "a", "b", "b", "b", "a", "a", "a", "b", "b", "b"], age = [4, 6, 3, 5, 7, 4, 5, 7, 4, 4, 6, 3, 6, 5, 4, 5, 7, 5])
> let m = f.anova(gain ~ category(dose) * category(diet) + age)
> m.posthoc(term: dose)
posthoc · Tukey HSD · dose · gain ~ category(dose) * category(diet) + age
each row tests H0: the two marginal means of the population are equal, against H1: they differ

  comparison   difference   std.error   t value   p value   95% CI
  low − mid        -0.900       0.194     -4.63     0.002   [-1.425, -0.375]
  low − top        -1.440       0.198     -7.29    <0.001   [-1.974, -0.907]
  mid − top        -0.540       0.198     -2.73     0.047   [-1.074, -0.007]

Tukey's HSD over 3 comparisons of 3 marginal means: p-values and intervals from the studentized range, family-wise
each mean is the model's prediction for a level of dose, averaged with equal weight over the levels of diet, with the numeric variables at their means (age = 5)
residual variance of the model on 11 degrees of freedom; N = 18
note: `dose` is also part of `dose:diet`: its differences are averages over the levels of `diet` and may not be the same at each level (line 10)
> m.posthoc(term: "dose:diet", method: holm)
posthoc · Holm · dose:diet · gain ~ category(dose) * category(diet) + age
each row tests H0: the two marginal means of the population are equal, against H1: they differ

  comparison      difference   std.error   t value   p value
  low:a − low:b        0.020       0.284      0.07     0.946
  low:a − mid:a       -0.414       0.284     -1.46     0.574
  low:a − mid:b       -1.367       0.275     -4.97     0.005
  low:a − top:a       -0.731       0.279     -2.62     0.165
  low:a − top:b       -2.130       0.290     -7.33    <0.001
  low:b − mid:a       -0.433       0.275     -1.58     0.574
  low:b − mid:b       -1.386       0.284     -4.89     0.005
  low:b − top:a       -0.751       0.276     -2.72     0.159
  low:b − top:b       -2.149       0.276     -7.79    <0.001
  mid:a − mid:b       -0.953       0.284     -3.36     0.057
  mid:a − top:a       -0.318       0.276     -1.15     0.574
  mid:a − top:b       -1.716       0.276     -6.22     0.001
  mid:b − top:a        0.635       0.279      2.28     0.219
  mid:b − top:b       -0.763       0.290     -2.63     0.165
  top:a − top:b       -1.398       0.279     -5.01     0.005

Holm over 15 comparisons of 6 marginal means: p-values adjusted family-wise; step-wise, so no intervals
each mean is the model's prediction for a cell of dose:diet, with the numeric variables at their means (age = 5)
residual variance of the model on 11 degrees of freedom; N = 18
> f.anova(gain ~ category(dose) + age).posthoc(method: bonferroni)
posthoc · Bonferroni · dose · gain ~ category(dose) + age
each row tests H0: the two marginal means of the population are equal, against H1: they differ

  comparison   difference   std.error   t value   p value   95% CI
  low − mid        -0.900       0.363     -2.48     0.080   [-1.887, 0.087]
  low − top        -1.441       0.368     -3.91     0.005   [-2.442, -0.440]
  mid − top        -0.541       0.368     -1.47     0.491   [-1.542, 0.460]

Bonferroni over 3 comparisons of 3 marginal means: each p-value multiplied by 3, each interval at the level 1 − 0.05/3
each mean is the model's prediction for a level of dose, with the numeric variables at their means (age = 5)
residual variance of the model on 14 degrees of freedom; N = 18
> b.p_values("a − b")
0.000608127
> b.p_values("a - b")
0.000608127
> b.statistic("b − c")
4.39931
> b.df("b − c")
11
> d.anova(gain, group: diet, equal_var: true).posthoc(level: 0.99)
posthoc · Tukey HSD · gain by diet
each row tests H0: the two population means are equal, against H1: they differ

  comparison   difference   std.error   t value   p value   99% CI
  a − b            -1.120       0.206     -5.44     0.001   [-1.869, -0.371]
  a − c            -0.160       0.218     -0.73     0.750   [-0.954, 0.634]
  b − c             0.960       0.218      4.40     0.003   [0.166, 1.754]

Tukey's HSD over 3 comparisons of 3 groups: p-values and intervals from the studentized range, family-wise
pooled within-group variance on 11 degrees of freedom; N = 14
> d.anova(gain, group: diet, equal_var: true).posthoc(method: sidak, level: 0.9).ci()
ci · 90%

  term     lower    upper
  a − b   -1.616   -0.624
  a − c   -0.686    0.366
  b − c    0.434    1.486
