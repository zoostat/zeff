# anova

## Summary
One-way analysis of variance across the groups of a variable

## Description
Whether the population mean of a numeric variable is the same in every group of `group:`.
Welch's F is the default, as R's `oneway.test`: it does not assume that the groups share a
variance, and its second degrees of freedom are fractional. `equal_var: true` gives the
classical F on the pooled within-group variance, as R's `aov` and Stata's `oneway`, with its
table of sums of squares. Either form prints the hypotheses, a table of the groups with their
counts, means and standard deviations, F with its degrees of freedom and the p-value, eta
squared, a decision at α = 0.05, and a line saying how F was computed. Rows with `NA` in the
variable or the grouping variable are dropped and counted in a note. Two groups are allowed;
with two, Welch's F is the square of Welch's t from `t_test`, with the same p-value.

## Arguments
- data: The frame.
- var: The numeric variable whose means are compared.
- group: The variable whose groups are compared, a Bool, Int, Str or Category variable; a
  group's label is its value label where it has one and `var=value` otherwise.
- equal_var: `false` for Welch's F, `true` for the classical F on the pooled variance.
- weights: Frequency weights, each row counted that many times, or analytic weights, which
  scale each group's variance; sampling weights are refused by the shared rule (Z0411).

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`): the statistic, its two degrees of freedom, the p-value, eta squared as the
effect size, and the hypotheses. Its own methods `statistic()`, `df()`, `effect_size()`,
`h0()`, `h1()` and `alternative()` give the figures as values.

## Details
With k groups of sizes nᵢ, means mᵢ and variances sᵢ², Welch's F weights each group by
wᵢ = nᵢ/sᵢ²: F = Σ wᵢ(mᵢ − m̃)² / ((k − 1)(1 + 2(k − 2)Λ)), where m̃ is the weighted grand mean,
Λ = Σ (1 − wᵢ/Σw)²/(nᵢ − 1) / (k² − 1), and the degrees of freedom are k − 1 and 1/(3Λ). The
classical F is the mean square between the groups over the mean square within them, on k − 1 and
N − k degrees of freedom. Eta squared is the sum of squares between the groups over the total
sum of squares in both forms, from the unweighted decomposition; it has no interval, since the
interval of an F effect size needs the noncentral F distribution, which `dist` does not have yet.
The p-value is the upper tail of `dist.f`, computed directly. Welch's test needs at least two
values with some variation in every group; the classical test needs more values than groups and
some variation within them. A grouping variable with fewer than two groups among the rows kept
is an error.

## References
Welch (1951), On the comparison of several mean values: an alternative approach, Biometrika 38,
330–336.
Fisher (1925), Statistical Methods for Research Workers, Oliver and Boyd, chapter 7.

## See also
t_test, kruskal_test, posthoc, p_adjust

## Examples
> let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])
> g.anova(s, group: site)
anova · Welch one-way · s by site
  H0: the population means of s are equal in all 3 groups of site          μ₁ = μ₂ = μ₃
  H1: at least one group of site has a different population mean of s      not all equal

  group    N    mean      sd
  site=1   5   2.840   0.288
  site=2   4   3.225   0.793
  site=3   5   2.820   0.736

F(2, 5.5) = 0.39   p = 0.693
eta squared = 0.092
decision at α = 0.05: fail to reject H0 (p = 0.693)
Welch's F over 3 groups, the variances not assumed equal; eta squared = SS between / SS total
> g.anova(s, group: site, equal_var: true)
anova · classical one-way · s by site
  H0: the population means of s are equal in all 3 groups of site          μ₁ = μ₂ = μ₃
  H1: at least one group of site has a different population mean of s      not all equal

  group    N    mean      sd
  site=1   5   2.840   0.288
  site=2   4   3.225   0.793
  site=3   5   2.820   0.736

  source           df      SS      MS
  between groups    2   0.447   0.223
  within groups    11   4.388   0.399
  ───────────────────────────────────
  total            13   4.834

F(2, 11) = 0.56   p = 0.587
eta squared = 0.092
decision at α = 0.05: fail to reject H0 (p = 0.587)
classical F over 3 groups on the pooled within-group variance; eta squared = SS between / SS total
> let wages = dataset(wages).add(lwage = log(wage))
> wages.anova(lwage, group: industry)
anova · Welch one-way · lwage by industry
  H0: the population means of lwage are equal in all 12 groups of industry         μ₁ = μ₂ = … = μ₁₂
  H1: at least one group of industry has a different population mean of lwage      not all equal

  group                       N    mean      sd
  Agriculture               528   3.004   0.460
  Mining                    199   3.190   0.410
  Utilities                 366   3.229   0.459
  Construction            1,457   3.244   0.478
  Manufacturing           2,543   3.238   0.475
  Retail                  2,836   3.151   0.479
  Transport                 962   3.175   0.462
  Finance                 1,544   3.279   0.503
  Services                2,612   3.097   0.492
  Education               1,669   3.198   0.482
  Health                  2,077   3.197   0.497
  Public administration   1,408   3.278   0.456

F(11, 3,489.4) = 31.60   p < 0.001
eta squared = 0.019
decision at α = 0.05: reject H0 (p < 0.001)
Welch's F over 12 groups, the variances not assumed equal; eta squared = SS between / SS total
note: 211 rows with NA in lwage or industry dropped (line 5)
> wages.anova(lwage, group: industry).effect_size()
0.0185603
note: 211 rows with NA in lwage or industry dropped (line 6)

