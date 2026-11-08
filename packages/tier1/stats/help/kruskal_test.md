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
A `TestResult`, with the result interface (`summary`, `p_values`, `n_obs`, `export`, `report`):
the statistic, its degrees of freedom, the p-value and the hypotheses.

## Details
Every value is ranked in the pooled sample, ties taking the average rank. With N values in k
groups whose rank sums are R₁ … R_k, H = 12/(N(N + 1)) Σ R_g²/n_g − 3(N + 1), divided by the tie
correction 1 − Σ(t³ − t)/(N³ − N) over the sizes t of the tied sets. Under H0, H follows
approximately a chi-squared distribution with k − 1 degrees of freedom, whose upper tail is the
p-value, as R's `kruskal.test` and Stata's `kwallis` compute it. A grouping variable with fewer
than two groups among the rows kept, or a variable whose values are all the same, is an error.

## References
Kruskal and Wallis (1952), Use of ranks in one-criterion variance analysis, Journal of the
American Statistical Association 47, 583–621.
Hollander, Wolfe and Chicken (2014), Nonparametric Statistical Methods, third edition, Wiley,
chapter 6.

## See also
anova, wilcoxon_test, p_adjust

## Examples
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
