# kruskal_test

## Summary
Kruskal–Wallis rank-sum test across several groups

## Description
The rank-based alternative to one-way `anova`: whether a variable's distribution is the same in
every group of `group:`, judged from the ranks of the pooled sample. It prints the hypotheses,
a table of the groups with their counts and medians, the statistic H with its degrees of freedom
and the p-value from the chi-squared approximation, a decision at α = 0.05, and a line saying
how the p-value was computed. Rows with `NA` in the variable or the grouping variable are dropped
and counted in a note. Two groups are allowed; with two, `wilcoxon_test` gives the same p-value
without a continuity correction, together with an estimate of the shift.

## Arguments
- data: The frame.
- var: The numeric variable whose distribution is compared.
- group: The variable whose groups are compared, a Bool, Int or Str variable; a group's label is
  its value label where it has one and `var=value` otherwise.
- weights: Frequency weights, each row repeated that many times; analytic weights are refused,
  since ranks cannot be scaled, and sampling weights by the shared rule (Z0411).

## Value
A `TestResult`, with the result interface (`summary`, `p_values`, `n_obs`, `export`, `report`): the
statistic, its degrees of freedom, the p-value and the hypotheses. Its own methods `statistic()`,
`df()`, `effect_size()`, `h0()`, `h1()` and `alternative()` give the figures as values.

## Missing values
A row with `NA` in the variable or in `group:` is left out, and a note gives the count. With `weights:`, a row whose weight is missing is left out as well. A group left with no row is not a group of the test.

## Details
Every value is ranked in the pooled sample, ties taking the average rank. With N values in k
groups whose rank sums are R₁ … R_k, H = 12/(N(N + 1)) Σ R_g²/n_g − 3(N + 1), divided by the tie
correction 1 − Σ(t³ − t)/(N³ − N) over the sizes t of the tied sets. Under H0, H follows
approximately a chi-squared distribution with k − 1 degrees of freedom, whose upper tail is the
p-value, as R's `kruskal.test` and Stata's `kwallis` compute it. A grouping variable with fewer
than two groups among the rows kept, or a variable whose values are all the same, is an error.

## A common mistake
Writing the groups as a formula, as `kruskal.test(y ~ g)` in R. The variable comes first and the groups go in `group:`:

> dataset(firms).kruskal_test(size ~ industry)
error[Z0401]: `var` of `kruskal_test` must be a Var, but this is a Formula
  = help: name a variable of the frame, without quotes

Write `kruskal_test(size, group: industry)`.

## Coming from R or Stata
| d.kruskal_test(y, group: g) | kruskal.test(y ~ g, data = d) | kwallis y, by(g) |

H is corrected for ties, as R's is. Stata's `kwallis` prints two statistics, without and with
the correction; the second is the one given here.

## Related tasks
- For the same comparison of means, use `anova`.
- For two groups, with an estimate of the shift between them, use `wilcoxon_test`.
- To adjust the p-values of tests made pair by pair, use `p_adjust`.
- To check whether the groups are equally spread, use `variance_test`.
- For the median of each group as a frame, use `aggregate`.

## References
Kruskal and Wallis (1952), Use of ranks in one-criterion variance analysis, Journal of the
American Statistical Association 47, 583–621.
Hollander, Wolfe and Chicken (2014), Nonparametric Statistical Methods, third edition, Wiley,
chapter 6.

## See also
anova, wilcoxon_test, p_adjust

## Examples
// Do three sites share one distribution? The ranks of the fourteen values, pooled, are compared.
> let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])
> g.kruskal_test(s, group: site)
kruskal_test · s by site
  H0: the distribution of s is the same in every group of site
  H1: values of s in at least one group of site tend to be larger or smaller than in another

  group    N   median
  site=1   5    2.900
  site=2   4    3.250
  site=3   5    2.800

H(2) = 0.77   p = 0.680
decision at α = 0.05: fail to reject H0 (p = 0.680)
chi-squared approximation over 3 groups, no ties

// On the shipped wages: the wage across the twelve industries in one year.
> dataset(wages).where(year == 2019).kruskal_test(wage, group: industry)
kruskal_test · wage by industry
  H0: the distribution of wage is the same in every group of industry
  H1: values of wage in at least one group of industry tend to be larger or smaller than in another

  group                     N   median
  Agriculture              44   24.740
  Mining                   15   33.690
  Utilities                38   27.160
  Construction            125   29.170
  Manufacturing           204   29.500
  Retail                  231   26.230
  Transport                83   27.500
  Finance                 129   32.360
  Services                204   24.305
  Education               130   26.610
  Health                  166   26.565
  Public administration   123   29.310

H(11) = 29.95   p = 0.002
decision at α = 0.05: reject H0 (p = 0.002)
chi-squared approximation over 12 groups, ranks averaged over ties and the tie correction applied
note: 26 rows with NA in wage or industry dropped (line 3)

// Data given as counts: each row is a score with the number of people who gave it.
> let scores = frame(score = [1, 2, 3, 4, 5, 1, 2, 3, 4, 5], arm = ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b"], count = [3, 8, 14, 9, 4, 6, 11, 12, 5, 2])
> scores.kruskal_test(score, group: arm, weights: count, weight_type: frequency)
kruskal_test · score by arm
  H0: the distribution of score is the same in every group of arm
  H1: values of score in at least one group of arm tend to be larger or smaller than in another

  group    N   median
  a       38    3.000
  b       36    3.000

H(1) = 3.32   p = 0.069
decision at α = 0.05: fail to reject H0 (p = 0.069)
chi-squared approximation over 2 groups, ranks averaged over ties and the tie correction applied

// The statistic, its degrees of freedom and the p-value as values.
> let k = g.kruskal_test(s, group: site)
> k.statistic()
> k.df()
> k.p_values()
0.771429
2
0.679965
