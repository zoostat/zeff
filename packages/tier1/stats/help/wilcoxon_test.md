# wilcoxon_test

## Summary
Wilcoxon signed-rank and rank-sum tests

## Description
The rank-based alternative to `t_test`, in the same three forms: the signed-rank test of one
variable against `mu:`, the rank-sum test (Mann–Whitney) of a variable between the two levels of
`group:`, and the signed-rank test of paired differences with `paired: true`. Each prints the
hypotheses it tests; the counts, with the medians of two groups; the Hodges–Lehmann estimate
(the pseudomedian, or the location shift between two groups) with its confidence interval; the
statistic (V or W) and the p-value; a decision at α = 1 − level; and a line saying how the
p-value was computed.

Below 50 observations in a sample (in each group for the rank-sum test) the p-value is exact,
from the distribution of the statistic counted in integers, over the tied ranks when there are
ties or differences of zero; otherwise it comes from the normal approximation with the
continuity correction, which `correct: false` drops. `exact:` overrides the rule either way.
This is what R 4.6.0's `wilcox.test` does; Stata's `signrank` and `ranksum` always approximate.
Rows with `NA` in a tested variable are dropped and counted in a note.

## Arguments
- data: The frame.
- var: The variable tested; in the paired form, the first measurement.
- b: The second measurement of the paired form, with `paired: true`; omitted otherwise.
- mu: The hypothesised centre of symmetry, or the hypothesised shift of the first group.
- exact: `true` for the exact p-value and interval, `false` for the normal approximation;
  omitted, exact below 50 observations.
- correct: Apply the continuity correction in the normal approximation.
- group: The variable whose two groups are compared; one level or more than two is an error
  that suggests `kruskal_test` (Z0412).
- paired: `true` selects the paired form, `wilcoxon_test(a, b, paired: true)`.
- alternative: The alternative tested, which H1, the p-value and the interval follow: `less`
  means values tend to lie below `mu`, or the first group's values tend to be smaller.
- level: Confidence level; the exact interval's actual coverage is at least the level asked for.
- weights: Frequency weights, each row repeated that many times; analytic weights are refused,
  since ranks cannot be scaled, and sampling weights by the shared rule (Z0411).

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`): the statistic, the p-value, the estimate with its interval, and the hypotheses.
Its own methods `statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and `alternative()` give the
figures as values.

## Details
V is the sum of the ranks of the positive differences among the absolute differences; W is the
sum of the first group's ranks in the pooled sample less m(m + 1)/2. Ties take the average rank.
In the exact test a difference of zero keeps its place among the ranks and is set aside from the
sum, as R does; in the approximation it is dropped before ranking. The exact two-sided p-value
is twice the smaller tail, at most 1. The approximation's variance subtracts the usual tie term,
Σ(t³ − t)/48 for the signed-rank and Σ(t³ − t)/(N(N − 1)) scaled for the rank-sum. The estimate
is the median of the Walsh averages (xᵢ + xⱼ)/2 or of the pairwise differences xᵢ − yⱼ. The
exact interval takes its bounds from those order statistics at the quantiles of the exact
distribution, searched among them when there are ties; the approximate interval's bounds are the
shifts at which the standardised statistic meets the normal quantile, as R's `wilcox.test`
with `conf.int = TRUE` finds them. A one-sided alternative gives a one-sided interval.

## References
Wilcoxon (1945), Individual comparisons by ranking methods, Biometrics Bulletin 1, 80–83.
Mann and Whitney (1947), On a test of whether one of two random variables is stochastically
larger than the other, Annals of Mathematical Statistics 18, 50–60.
Hodges and Lehmann (1963), Estimates of location based on rank tests, Annals of Mathematical
Statistics 34, 598–611.
Hollander, Wolfe and Chicken (2014), Nonparametric Statistical Methods, third edition, Wiley,
chapters 3 and 4.

## See also
t_test, kruskal_test, anova

## Examples
> let d = frame(x = [1.83, 0.50, 1.62, 2.48, 1.68, 1.88, 1.55, 3.06, 1.30], y = [0.878, 0.647, 0.598, 2.05, 1.06, 1.29, 1.06, 3.14, 1.29])
> d.wilcoxon_test(x, mu: 1)
wilcoxon_test · signed rank · x
  H0: the distribution of x is symmetric about 1
  H1: the distribution of x is not symmetric about 1; values tend to lie on one side of it

N = 9
pseudomedian = 1.725   95% CI [1.165, 2.370]
V = 43.00   p = 0.012
decision at α = 0.05: reject H0 (p = 0.012)
exact p-value, no ties
> d.wilcoxon_test(x, y, paired: true, alternative: greater)
wilcoxon_test · signed rank · paired · x and y
  H0: the distribution of the paired differences x − y is symmetric about 0
  H1: the paired differences x − y tend to lie above 0

N = 9 pairs
pseudomedian of differences (x − y) = 0.460   95% CI [0.175, Inf)
V = 40.00   p = 0.020
decision at α = 0.05: reject H0 (p = 0.020)
exact p-value, no ties
> let g = frame(s = [0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.64, 0.73, 1.46, 1.15, 0.88, 0.90, 0.74, 1.21], grp = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2])
> g.wilcoxon_test(s, group: grp)
wilcoxon_test · rank sum · s by grp
  H0: the distribution of s is the same in grp=1 and grp=2
  H1: values of s in grp=1 tend to be larger or smaller than in grp=2

  group    N   median
  grp=1   10    1.415
  grp=2    5    0.900

location shift (grp=1 − grp=2) = 0.305   95% CI [-0.150, 0.760]
W = 35.00   p = 0.254
decision at α = 0.05: fail to reject H0 (p = 0.254)
exact p-value, no ties
